
target remote :3333

monitor arm semihosting enable

set history save
set verbose off
set print pretty on
set print array off
set print array-indexes on
set python print-stack full

load
