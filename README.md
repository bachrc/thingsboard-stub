# Thingsboard Stub

This tool permits you to simulate sensors via MQTT to Thingsboard.

This stub for now only permits to simulate temperature-like sensors.

## Usage

```shell script
Thingsboard Stub 0.1
bachrc <8.bachrc@gmail.com>
It simulates a temperature sensor for your Thingsboard needs !

USAGE:
    thingsboard-stub --hostname <HOSTNAME> --name <NAME> --port <PORT> --secret <SECRET_TOKEN> --temperature <TEMPERATURE_NAME>...

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --hostname <HOSTNAME>                  The hostname of the MQTT broker used by Thingsboard
    -n, --name <NAME>                          The name of the industruino
    -p, --port <PORT>                          The port of the MQTT Broker used by Thingsboard
    -s, --secret <SECRET_TOKEN>                The Secret Token used to authenticate the Industruino
    -t, --temperature <TEMPERATURE_NAME>...    The name of your temperature sensor
```
