#!bin/ksh
# Build script for receiptd. Does all the necessary pre-configuration for the
# installation of receiptd. Must be run as root.

useradd -b /nonexistent -c "PDF Receipt Making Daemon" -g=uid -L daemon -s /sbin/nologin -r 100..999 receiptd

mkdir -m 0755 -p /var/log/receiptd /var/receiptd
chown receiptd:receiptd /var/log/receiptd /var/receiptd

cp -R ../fonts /var/receiptd/
chmod -R 755 /var/receiptd

cp receiptd.rc.d /etc/rc.d/receiptd
chmod 0555 /etc/rc.d/receiptd

cp receiptd.conf /etc/receiptd.conf
chmod 0664 /etc/receiptd.conf

