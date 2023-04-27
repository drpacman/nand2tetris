// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed. 
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.
// Put your code here.

(LOOP)
// set loc to contain the screen size
@8192 // the screen size 32 bytes *256 
D=A
@LOC
M=D

// get the key input - 0 (no entry) means paint white
@KBD
D=M
@WHITE
D;JEQ

(BLACK)
@SCREEN
D=A
@LOC
M=M-1
A=M+D
M=-1
@LOC
D=M
@BLACK
D;JNE

@LOOP
0;JMP

(WHITE)
@SCREEN
D=A
@LOC
M=M-1
A=M+D
M=0
@LOC
D=M
@WHITE
D;JNE

@LOOP
0;JMP
