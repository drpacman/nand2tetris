// push constant 0// Push 0@0D=A@SPA=MM=D@SPM=M+1// pop local 0@LCLA=MD=A@R13M=D@SPM=M-1A=MD=M@R13A=MM=D// (LOOP_START)(LOOP_START)// push argument 0@ARGA=MD=M@SPA=MM=D@SPM=M+1// push local 0@LCLA=MD=M@SPA=MM=D@SPM=M+1// add// add@SPA=MA=A-1D=MA=A-1M=D+M@SPM=M-1// pop local 0@LCLA=MD=A@R13M=D@SPM=M-1A=MD=M@R13A=MM=D// push argument 0@ARGA=MD=M@SPA=MM=D@SPM=M+1// push constant 1// Push 1@1D=A@SPA=MM=D@SPM=M+1// sub// sub@SPA=MA=A-1D=MA=A-1M=M-D@SPM=M-1// pop argument 0@ARGA=MD=A@R13M=D@SPM=M-1A=MD=M@R13A=MM=D// push argument 0@ARGA=MD=M@SPA=MM=D@SPM=M+1// if-goto LOOP_START@SPM=M-1A=MD=M@LOOP_STARTD;JNE// push local 0@LCLA=MD=M@SPA=MM=D@SPM=M+1