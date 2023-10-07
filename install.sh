#!/bin/bash

config_directory="/etc/argonone"
config_file="argon_services_config.yaml"

fan_binary_name="argon_fan"
fan_service="$fan_binary_name.service"

shutdown_button_binary_name="argon_shutdown_button"
shutdown_button_service="$shutdown_button_binary_name.service"

shutdown_binary_name="argon_shutdown"


# Check if raspi-config is available, this package should be available in most raspberry pi distributions.
command -v raspi-config &> /dev/null
if [ $? -eq 0 ]
then
	# Enable i2c and serial
	echo "Enabling i2c and serial..."
	sudo raspi-config nonint do_i2c 0
	sudo raspi-config nonint do_serial 2
fi

# Check if the config directory exits
if [ ! -d $config_directory ]; then
  # If it does not exist, create the folder
  echo "Creating config directory: $config_directory"
  sudo mkdir -p $config_directory
  sudo chmod 755 $config_directory
fi


# Stop running services
if systemctl is-active --quiet $fan_service; then
    echo "Stopping fan service..."
    sudo systemctl stop $fan_service
    sudo systemctl disable $fan_service
fi

if systemctl is-active --quiet $shutdown_button_service; then
  echo "Stopping power button service..."
  sudo systemctl stop $shutdown_button_service
  sudo systemctl disable $shutdown_button_service
fi


# Copy configuration file if it exists
if test -f "argon_services_config.yaml"; then
  echo "Moving config file to $config_directory/$config_file"
  sudo cp ./$config_file $config_directory
fi


echo "Copying executables..."

# Copy fan executable
sudo chmod 755 ./$fan_binary_name
sudo cp ./$fan_binary_name /usr/bin/

# Copy power button executable
sudo chmod 755 ./$shutdown_button_binary_name
sudo cp ./$shutdown_button_binary_name /usr/bin/

# Copy shutdown executable
sudo chmod 755 ./$shutdown_binary_name
sudo cp ./$shutdown_binary_name /lib/systemd/system-shutdown/


# Copy fan service file
sudo chmod 644 ./$fan_service
sudo cp ./$fan_service /lib/systemd/system/

# Copy power button service file
sudo chmod 644 ./$shutdown_button_service
sudo cp ./$shutdown_button_service /lib/systemd/system/


echo "Creating and running systemd services..."

# Enable and start services
sudo systemctl enable /lib/systemd/system/$fan_service
sudo systemctl start $fan_service
sudo systemctl enable /lib/systemd/system/$shutdown_button_service
sudo systemctl start $shutdown_button_service


echo "Done!"