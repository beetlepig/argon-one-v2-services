#!/bin/bash

fan_service="argon_fan.service"
shutdown_button_service="argon_shutdown_button.service"


# Check if raspi-config is available, this package should be available in most raspberry pi distributions.
command -v raspi-config &> /dev/null
if [ $? -eq 0 ]
then
	# Enable i2c and serial
	echo "Enabling i2c and serial..."
	sudo raspi-config nonint do_i2c 0
	sudo raspi-config nonint do_serial 2
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
  echo "Moving config file to /etc/argon_services_config.yaml"
  sudo cp ./argon_services_config.yaml /etc/
fi


echo "Copying executables..."

# Copy fan executable
sudo chmod 755 ./argon_fan
sudo cp ./argon_fan /usr/bin/

# Copy shutdown executable
sudo chmod 755 ./argon_shutdown
sudo cp ./argon_shutdown /lib/systemd/system-shutdown/

# Copy power button executable
sudo chmod 755 ./argon_shutdown_button
sudo cp ./argon_shutdown_button /usr/bin/


# Copy fan service file
sudo chmod 644 ./argon_fan.service
sudo cp ./argon_fan.service /lib/systemd/system/

# Copy power button service file
sudo chmod 644 ./argon_shutdown_button.service
sudo cp ./argon_shutdown_button.service /lib/systemd/system/


echo "Creating and running systemd services..."

# Enable and start services
sudo systemctl enable /lib/systemd/system/$fan_service
sudo systemctl start $fan_service
sudo systemctl enable /lib/systemd/system/$shutdown_button_service
sudo systemctl start $shutdown_button_service


echo "Done!"