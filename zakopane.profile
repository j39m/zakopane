# This firejail profile is distributed with zakopane.

quiet

include zakopane.local

include disable-exec.inc
#include disable-shell.inc
include disable-write-mnt.inc

dbus-system none
dbus-user none

net none

read-only ${HOME}
