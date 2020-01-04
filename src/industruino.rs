use crate::configuration::Config;
use crate::messages::{IncomingRPCRequest, IncomingRequest, OutgoingMessage};
use crate::temperature::Temperature;
use crossbeam_channel::Sender;
use rumqtt::mqttoptions::SecurityOptions::UsernamePassword;
use rumqtt::Notification::Publish;
use rumqtt::{MqttClient, MqttOptions, Notification, QoS, Receiver, ReconnectOptions};
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use OutgoingMessage::{AnswerGetValue, Telemetry};

pub struct Industruino {
    name: String,
    secret_token: String,

    mqtt_client: Option<MqttClient>,
    notifications: Option<Receiver<Notification>>,

    incoming_notification_sender: HashMap<String, Sender<IncomingRequest>>,

    outgoing_sender: crossbeam_channel::Sender<OutgoingMessage>,
    outgoing_receiver: crossbeam_channel::Receiver<OutgoingMessage>,
}

impl Industruino {
    pub fn new(name: String, secret_token: String) -> Industruino {
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::unbounded();

        return Industruino {
            name,
            secret_token,
            mqtt_client: None,
            notifications: None,
            incoming_notification_sender: HashMap::new(),
            outgoing_receiver,
            outgoing_sender,
        };
    }

    pub fn create_attached_temperature_sensor(
        &mut self, libelle: &str, initial_temperature: f32,
    ) -> Temperature {
        let (incoming_request_sender, incoming_request_receiver) = crossbeam_channel::unbounded();

        self.incoming_notification_sender
            .insert(String::from(libelle), incoming_request_sender);

        Temperature::new(
            initial_temperature,
            libelle,
            self.outgoing_sender.clone(),
            incoming_request_receiver,
        )
    }

    pub fn connect(self, hostname: &str, port: u16) -> Industruino {
        let mqtt_options = MqttOptions::new(&self.name, hostname, port)
            .set_security_opts(UsernamePassword(
                String::from(&self.secret_token),
                String::from(""),
            ))
            .set_keep_alive(10)
            .set_inflight(3)
            .set_request_channel_capacity(3)
            .set_reconnect_opts(ReconnectOptions::Always(10))
            .set_clean_session(false);

        let (mqtt_client, notifications) = MqttClient::start(mqtt_options).unwrap();

        Industruino {
            mqtt_client: Some(mqtt_client),
            notifications: Some(notifications),
            ..self
        }
    }

    fn send_telemetry(mqtt_client: &mut MqttClient, values: HashMap<String, String>) {
        let payload = serde_json::to_string(&values).expect("Serialization of the payload failed.");

        mqtt_client
            .publish(
                "v1/devices/me/telemetry",
                QoS::AtLeastOnce,
                false,
                payload.clone(),
            )
            .expect("Error while sending telemetry value");

        println!("Valeur du capteur envoyé : {}", &payload)
    }

    fn dispatch_incoming_message(
        sensors_senders: &HashMap<String, Sender<IncomingRequest>>, message: rumqtt::Publish,
    ) {
        let received_message = String::from_utf8(message.payload.to_vec())
            .expect("Erreur lors du décodage du message entrant");

        println!("Le message qu'on a reçu, c'est lui : {}", received_message);

        let deserialized_payload: IncomingRPCRequest =
            serde_json::from_str(received_message.as_str())
                .expect("Error while parsing json from incoming message.");

        let method = deserialized_payload.method;
        let params = deserialized_payload.params;
        println!("Incoming RPC Request, method : {}", method);

        let request_elements: Vec<&str> = method.split("-").collect();
        if request_elements.len() < 2 {
            return;
        }
        let request_type: &str = request_elements.get(0).expect("Invalid method format");
        let targeted_sensor: &str = request_elements.get(1).expect("Invalid method format");

        let sensor_incoming_sender = sensors_senders
            .get(targeted_sensor)
            .expect("Le capteur renseigné a échoué");

        let request_number: u32 = message
            .topic_name
            .split("/")
            .collect::<Vec<&str>>()
            .last()
            .and_then(|string_id| string_id.parse::<u32>().ok())
            .expect("échec de la récupération");

        match request_type.as_ref() {
            "get" => sensor_incoming_sender
                .send(IncomingRequest::GetValue(request_number))
                .unwrap(),
            "set" => sensor_incoming_sender
                .send(IncomingRequest::SetValue(request_number, params.unwrap()))
                .unwrap(),
            _ => (),
        }
    }

    fn send_outgoing_message(mqtt_client: &mut MqttClient, request_number: u32, message: String) {
        mqtt_client
            .publish(
                format!("v1/devices/me/rpc/response/{}", request_number),
                QoS::AtLeastOnce,
                false,
                message.clone(),
            )
            .expect("Error while sending telemetry value");

        println!(
            "La réponse à la requête {} a bien été envoyée : {}",
            request_number, message
        );
    }

    pub fn run(&mut self) {
        let outgoing_messages = &self.outgoing_receiver;
        let incoming_messages = self.notifications.as_ref().unwrap();

        let mqtt_client = &mut self
            .mqtt_client
            .as_mut()
            .expect("MQTT Client is not connected");

        mqtt_client
            .subscribe("v1/devices/me/rpc/request/+", QoS::AtLeastOnce)
            .expect("Subscribe didn't go well.");

        loop {
            select! {
                recv(outgoing_messages) -> message => match message.unwrap() {
                    Telemetry(telemetry_message) => Self::send_telemetry(mqtt_client, telemetry_message),
                    AnswerGetValue(request_number, value) => Self::send_outgoing_message(mqtt_client, request_number, value),
                    _ => println!("Un message sortant inconnu est arrivé. On en fait rien.")
                },
                recv(incoming_messages) -> notification => match notification.unwrap() {
                    Publish(message) => Self::dispatch_incoming_message(&self.incoming_notification_sender, message),
                    _ => println!("Un message entrant inconnu est arrivé. On en fait rien.")
                }
            }
        }
    }
}
