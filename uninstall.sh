#!/bin/bash

config_directory="/etc/argonone"

fan_binary_name="argon_fan"
fan_service="$fan_binary_name.service"

shutdown_button_binary_name="argon_shutdown_button"
shutdown_button_service="$shutdown_button_binary_name.service"

shutdown_binary_name="argon_shutdown"


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


# Check if the config directory exits
if [ -d $config_directory ]; then
  # If it does exist, remove the folder
  sudo rm -rf $config_directory
  echo "Config directory $config_directory has been deleted."
fi


# Delete fan executable
if [ -f /usr/bin/$fan_binary_name ]; then
    sudo rm -f /usr/bin/$fan_binary_name
    echo "File /usr/bin/$fan_binary_name has been deleted."
fi

# Delete power button executable
if [ -f /usr/bin/$shutdown_button_binary_name ]; then
    sudo rm -f /usr/bin/$shutdown_button_binary_name
    echo "File /usr/bin/$shutdown_button_binary_name has been deleted."
fi

# Delete shutdown executable
if [ -f /lib/systemd/system-shutdown/$shutdown_binary_name ]; then
    sudo rm -f /lib/systemd/system-shutdown/$shutdown_binary_name
    echo "File /lib/systemd/system-shutdown/$shutdown_binary_name has been deleted."
fi


# Delete fan service
if [ -f /lib/systemd/system/$fan_service ]; then
    sudo rm -f /lib/systemd/system/$fan_service
    echo "Service file /lib/systemd/system/$fan_service has been deleted."
fi

# Delete power button service
if [ -f /lib/systemd/system/$shutdown_button_service ]; then
    sudo rm -f /lib/systemd/system/$shutdown_button_service
    echo "Service file /lib/systemd/system/$shutdown_button_service has been deleted."
fi


echo "Done!"