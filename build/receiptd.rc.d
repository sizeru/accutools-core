#!/bin/ksh

daemon="/usr/local/sbin/receiptd" 
. /etc/rc.d/rc.subr

rc_configtest() 
{
        ${daemon} -n
}

rc_cmd $1