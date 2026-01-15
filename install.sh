#!/bin/bash

# If not running as root, re-run the script with sudo
if [ "$EUID" -ne 0 ]; then
  echo "Privileges required. Requesting sudo..."
  exec sudo "$0" "$@"
fi

config_directory="/etc/argonone"
config_file="argon_services_config.yaml"

fan_binary_name="argon_fan"
fan_service="$fan_binary_name.service"

shutdown_button_binary_name="argon_shutdown_button"
shutdown_button_service="$shutdown_button_binary_name.service"

shutdown_binary_name="argon_shutdown"

set_config_var() {
  awk -v key="$1" -v value="$2" '
    $0 ~ "^#?[[:space:]]*" key "=" {
      print key "=" value
      made_change=1
      next
    }
    { print }
    END { if (!made_change) print key "=" value }
  ' "$3" > "$3.bak" && mv "$3.bak" "$3"
}

do_i2c() {
  if [ -e /boot/firmware/config.txt ] ; then
    FIRMWARE=/firmware
  else
    FIRMWARE=
  fi
  CONFIG=/boot${FIRMWARE}/config.txt

  SETTING=on


  set_config_var dtparam=i2c_arm $SETTING $CONFIG

  sed /etc/modules -i -e "s/^#[[:space:]]*\(i2c[-_]dev\)/\1/"
  if ! grep -q "^i2c[-_]dev" /etc/modules; then
    printf "i2c-dev\n" >> /etc/modules
  fi

  dtparam i2c_arm=$SETTING
  modprobe i2c-dev
}

enable_services() {
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
}

do_i2c
enable_services



echo "Done!"