LCON 1 01 ;
LMEM 0 "string_end" ;
LMEM 3 "print_pointer" ;
LMEM 2 "string_start" : "print_start" : "print_pointer" ;
CHAR 2 ;
ADDR 3 31 ;
STOR 3 "print_pointer" ;
JUMP 3 "halt" ;
JUMP 0 "print_start" ;
HALT 0 : "halt" ;
DATA "Hello, world!" : "string_start" : "string_end" ;
