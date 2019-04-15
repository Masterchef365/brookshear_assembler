LCON 1 01 ; Set reg1 to 1, used in pointer incrementing
LMEM 0 "string_end" : "restart" : "print_pointer_pointer" ; 
LMEM 3 "print_pointer" ;
LMEM 2 "string_start" : "print_start" : "print_pointer" ;
CHAR 2 ;
ADDR 3 31 ;
STOR 3 "print_pointer" ;
JUMP 3 "halt" ;
JUMP 0 "print_start" ;
LCON 5 "string2_end" ;
STOR 5 "print_pointer_pointer" ; yeah :qbitch
LCON 5 "string2_start" ;
STOR 5 "print_pointer" ;
JUMP 0 "restart" ;
HALT 0 : "halt" ;
DATA "Hello, world!" : "string_start" : "string_end" ;
DATA " This is another sentence!" : "string2_start" : "string2_end" ;
