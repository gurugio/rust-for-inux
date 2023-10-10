#! /bin/sh
module="rust_ldd03"
device="rust_ldd03"
mode="666"
group=0

function load() {
    insmod ./$module.ko $* || exit 1

    rm -f /dev/${device}[0-2]

    minor=$(awk -v device="$device"0 '$2==device {print $1}' /proc/misc)
    mknod /dev/${device}0 c 10 $minor
    minor=$(awk -v device="$device"1 '$2==device {print $1}' /proc/misc)
    mknod /dev/${device}1 c 10 $minor
    minor=$(awk -v device="$device"2 '$2==device {print $1}' /proc/misc)
    mknod /dev/${device}2 c 10 $minor

    chgrp $group /dev/$device[0-2]
    chmod $mode /dev/$device[0-2]
}

function unload() {
    rm -f /dev/${device}[0-2]
    rmmod $module || exit 1
}

arg=${1:-"load"}
case $arg in
    load)
        load ;;
    unload)
        unload ;;
    reload)
        ( unload )
        load
        ;;
    *)
        echo "Usage: $0 {load | unload | reload}"
        echo "Default is load"
        exit 1
        ;;
esac
