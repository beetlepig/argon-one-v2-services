# Case services for Argon One v2

A Rust implementation of the argon one case services, it aims to be a blazingly fast native replacement of the vanilla argon one python scripts, it provides the same functionality plus many customization options.

## Features

- Custom shutdown and reboot scripts.
- Fan configuration with hypertesis and an only way down option.
- Native and don't require installing GPIO or I2C packages.
- Linear interpolation between fan speeds to provide a smoother and precise fan curve.


## Installation

You can download the latest [Release](https://github.com/beetlepig/argon-one-v2-services/releases/latest) or use command line to download the package.

    curl -LJO https://github.com/beetlepig/argon-one-v2-services/releases/download/0.1.0/ArgonOneV2Services.tgz

Then you extract the package.

    tar -xvf ArgonOneV2Services.tgz

Go into the folder.

    cd ArgonOneV2Services

Run install script (you may need to give execution permission to the script).

    ./install.sh

## Uninstallation

Run the uninstall script.

    ./uninstall.sh


## How to update?
Just follow the installation step again, it will replace the old binaries with the new ones.

## Configuration file

The services use a YAML file to get the configuration options. All the configuration is done by editing the `argon_services_config.yaml` file.

Once installed, you can change the configuration by editing the config file previously copied to your system.

    sudo nano /etc/argonone/argon_services_config.yaml

Then, you can restart the systemd services to apply the changes (or you can reboot the system).

    sudo systemctl restart argon_fan.service

    sudo systemctl restart argon_shutdown_button.service


## **Fan configuration**

The fan configuration is specified in the `fan_config` key, and it looks like this:
```
fan_config:  
  interval: 10000  
  hysteresis:  
    amount: 4  
    only_way_down: true  
  matrix:  
    - [ 55, 10 ]  
    - [ 60, 40 ]  
    - [ 65, 100 ]
```
*Interval (Required):* This is the time in milliseconds between taking a new temperature measurement. A too small number will cause CPU to overhead. An interval between `2000` and `15000` is usually good. **Only integers allowed.**

*hysteresis (Required):* Hysteresis in a fan prevents it from rapidly switching on and off. This happens because the fan service has a delay or memory effect. For example, if the fan turns on when the raspberry gets too hot, it won't immediately turn off when the temperature drops slightly. Instead, it continues running until the temperature decreases a bit more. Likewise, when the temperature rises again, the fan won't instantly switch on; it waits until the temperature increases beyond a certain point. This delay ensures that the fan doesn't constantly cycle on and off rapidly, providing more stable and comfortable airflow.

- The `amount` value determines the temperature degrees to which the
  fan will be "delayed" before applying a new setting. A number between
  `4` and `10` is usually good. **Required and only integers allowed.**

- The `only_way_down` parameter determines whether hysteresis should
  only be applied when the temperature decreases. **Must be `true` or
  `false`.**

*Matrix (Required):* This is the temperature/speed matrix. Each entry consists of a pair of temperature and speed numbers, like `[ 55, 10 ]` in this example, this means that when the temperature is 55 degrees, the fan must run at 10% speed. The service will calculate a linear interpolation between each entry to ensure a smoother fan curve.

- **Both values must be integers between 0 and 100.**

- **A minimum of two temperature/speed entries are required for the algorithm to work.**

If you set any invalid value, the options will the fallback to a default configuration.

**How to update fan config once installed?**

Simply edit the `argon_services_config.yaml` located in `/etc/argon_services_config.yaml`.

    sudo nano /etc/argon_services_config.yaml

And then restart the systemd fan service.

    sudo systemctl restart argon_fan

## Shutdown button configuration

I added the possibility to run custom scripts when the bower button is pressed, for both actions, shutdown and reboot. This particularly useful for OS like Raspiblitz that needs to run a custom "shutdown" script in order to stop the node processes before doing the actual shutdown.

**Custom shutdown script (Optional)**

To add a custom shutdown script, you need to add the following configuration in the `argon_services_config.yaml` file. This parameter is optional, and it is not required to be declared in the config file.

```
shutdown_script:  
  location: "/home/admin/config.scripts/blitz.shutdown.sh"  
  args: []
```
The `location` key represents the file path of the script that will be executed when the shutdown action is triggered by pressing and holding the shutdown button for more than 3 seconds.

The `args` key is simply an array containing the arguments that will be supplied to the script.

The example above demonstrates the configuration for invoking the shutdown script in a Raspiblitz system.

**Custom reboot script (Optional)**

To add a custom reboot script, you need to add the following configuration in the `argon_services_config.yaml` file. This parameter is optional, and it is not required to be declared in the config file.

```
reboot_script:  
  location: "/home/admin/config.scripts/blitz.shutdown.sh"  
  args: ["reboot"]
```
The `location` key represents the file path of the script that will be executed when the reboot action is triggered by pressing the shutdown button twice.

The `args` key is simply an array containing the arguments that will be supplied to the script.

The example above demonstrates the configuration for invoking the shutdown script with a `reboot` argument in a Raspiblitz system.

**Example of a complete configuration**

```
fan_config:  
  interval: 10000  
  hysteresis:  
    amount: 4  
    only_way_down: true  
  matrix:  
    - [ 55, 10 ]  
    - [ 60, 40 ]  
    - [ 65, 100 ]  
shutdown_script:  
  location: "/home/admin/config.scripts/blitz.shutdown.sh"  
  args: []  
reboot_script:  
  location: "/home/admin/config.scripts/blitz.shutdown.sh"  
  args: ["reboot"]
```

## Compatibility

This should work out of the box in many Debian based distro for raspberry pi, I have tested the following:

- Raspberry PI OS
- Ubuntu 22.04
- Umbrel
- Raspiblitz

Right now, the installation script heavily depends on the `raspi-config` package in order to enable I2C and Serial. If your distribution does not work well with that package, you may need to find another way to enable I2C and Serial. The main binaries should work just fine once I2C and Serial are enabled.

If you are having trouble making this work on your distribution, I recommend checking out this [project](https://gitlab.com/DarkElvenAngel/argononed), which has excellent distribution support.

## Soon...

- More distro testing and support.
- An easier way to configure and change fan parameters.
- An easier way to install and update the services.
- An uninstall script.

## Acknowledgments

- [JhnW](https://github.com/JhnW/ArgonOne-Native-Fan-Controller) For the basics on how to implement this in Rust.
- [DarkElvenAngel](https://gitlab.com/DarkElvenAngel/argononed) For inspiration on which features to implement.
- [okunze](https://github.com/okunze/Argon40-ArgonOne-Script) For backup the official script and documentation.

**Copyright Notice:** The name "Argon" does not belong to me, and I am not affiliated with it in any way. This software repository is independent and not associated with any entity.
