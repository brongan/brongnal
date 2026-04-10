#!/bin/bash
set -e

# Hardening sysctl
sysctl -w net.ipv4.ip_forward=0

# Prepare and mount data disk
DEVICE_NAME="/dev/disk/by-id/google-data-disk"
MOUNT_POINT="/mnt/disks/gossamer"

mkdir -p $MOUNT_POINT

# Format if not already formatted
if ! blkid $DEVICE_NAME; then
  mkfs.ext4 $DEVICE_NAME
fi

# Mount with restrictive options
mount -o noexec,nosuid,nodev $DEVICE_NAME $MOUNT_POINT

# Adjust permissions for the container user (UID 1000)
chown 1000:1000 $MOUNT_POINT

# The container launch will be handled by the launcher binary 
# which will be configured via a systemd unit or COS cloud-config.
# For now, we'll just set up the disk.
