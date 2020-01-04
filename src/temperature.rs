use crate::messages::{IncomingRequest, OutgoingMessage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use IncomingRequest::{GetValue, SetValue};
use OutgoingMessage::{AnswerGetValue, Telemetry};

pub struct Temperature {
    pub temperature: Arc<Mutex<f32>>,
    pub libelle: Arc<String>,
    pub outgoing_sender: Arc<crossbeam_channel::Sender<OutgoingMessage>>,
    pub incoming_request_listener: Arc<crossbeam_channel::Receiver<IncomingRequest>>,
}

impl Temperature {
    pub fn new(
        initial_temperature: f32, libelle: &str,
        outgoing_sender: crossbeam_channel::Sender<OutgoingMessage>,
        incoming_request_listener: crossbeam_channel::Receiver<IncomingRequest>,
    ) -> Temperature {
        Temperature {
            temperature: Arc::new(Mutex::new(initial_temperature)),
            libelle: Arc::new(String::from(libelle)),
            outgoing_sender: Arc::new(outgoing_sender),
            incoming_request_listener: Arc::new(incoming_request_listener),
        }
    }

    pub fn run(&self) {
        let temperature = self.temperature.clone();
        let libelle = self.libelle.clone();
        let outgoing_sender = self.outgoing_sender.clone();

        thread::spawn(move || loop {
            Self::send_telemetry_value(
                libelle.clone(),
                temperature.clone(),
                outgoing_sender.clone(),
            );

            thread::sleep(Duration::from_secs(2))
        });

        let temperature = self.temperature.clone();
        let libelle = self.libelle.clone();
        let outgoing_sender = self.outgoing_sender.clone();
        let incoming_request_listener = self.incoming_request_listener.clone();

        thread::spawn(move || {
            Self::catch_incoming_requests(
                temperature,
                libelle,
                outgoing_sender,
                incoming_request_listener,
            )
        });
    }

    fn catch_incoming_requests(
        shared_temperature: Arc<Mutex<f32>>, libelle: Arc<String>,
        outgoing_sender: Arc<crossbeam_channel::Sender<OutgoingMessage>>,
        incoming_request_listener: Arc<crossbeam_channel::Receiver<IncomingRequest>>,
    ) {
        loop {
            select! {
                recv(incoming_request_listener) -> request => match request.unwrap() {
                    GetValue(request_number) => {
                        println!("J'ai reçu un getvalue, youpi");
                        let temperature: String = shared_temperature.lock().unwrap().to_string();
                        outgoing_sender.send(AnswerGetValue(request_number, temperature)).unwrap();
                    },
                    SetValue(request_number, new_value) => {
                        println!("J'ai reçu un setvalue, pour qu'on me mette la valeur : {}", &new_value);
                        let new_temperature = new_value.parse::<f32>().expect("Valeur reçue incorrecte");
                        {
                            let mut temperature = shared_temperature.lock().unwrap();
                            *temperature = new_temperature;
                            outgoing_sender.send(AnswerGetValue(request_number, temperature.to_string())).unwrap();
                        }
                        Self::send_telemetry_value(libelle.clone(), shared_temperature.clone(), outgoing_sender.clone());
                    }
                }
            }
        }
    }

    fn send_telemetry_value(
        libelle: Arc<String>, temperature: Arc<Mutex<f32>>,
        outgoing_sender: Arc<crossbeam_channel::Sender<OutgoingMessage>>,
    ) {
        let mut values: HashMap<String, String> = HashMap::new();
        let temperature = temperature.lock().unwrap();

        values.insert((*libelle).clone(), temperature.to_string());

        drop(temperature);

        outgoing_sender
            .send(Telemetry(values))
            .expect("Echec lors de l'envoi des données de télémétrie");
    }
}
