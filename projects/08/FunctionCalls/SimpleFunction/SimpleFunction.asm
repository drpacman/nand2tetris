@256D=A@SPM=D// call Sys.init 0// push ret_0@0D=A@R14M=D// set R15=Sys.init@Sys.initD=M@R15M=D@ret_0D=A// goto call_helper@call_helper0;JMP(ret_0)(restore_stack_and_return)// set R13=LCL@LCLD=M@R13M=D@5D=A@R13M=M-D// set R14=*R13@R13A=MD=M@R14M=D@SPM=M-1A=MD=M@ARGA=MM=D@ARGA=MD=A+1@SPM=D@R13M=M+1// set LCL=*R13@R13A=MD=M@LCLM=D@R13M=M+1// set ARG=*R13@R13A=MD=M@ARGM=D@R13M=M+1// set THIS=*R13@R13A=MD=M@THISM=D@R13M=M+1// set THAT=*R13@R13A=MD=M@THATM=D@R14A=M0;JMP(call_helper)@SPA=MM=D@SPM=M+1// Push LCL@LCLD=M@SPA=MM=D@SPM=M+1// Push ARG@ARGD=M@SPA=MM=D@SPM=M+1// Push THIS@THISD=M@SPA=MM=D@SPM=M+1// Push THAT@THATD=M@SPA=MM=D@SPM=M+1// Push SP@SPD=M@SPA=MM=D@SPM=M+1// Push 5@5D=A@SPA=MM=D@SPM=M+1// sub@SPA=MA=A-1D=MA=A-1M=M-D@SPM=M-1@R14D=M@SPA=MM=D@SPM=M+1// sub@SPA=MA=A-1D=MA=A-1M=M-D@SPM=M-1// pop ARG@SPM=M-1A=MD=M@ARGM=D// set LCL=SP@SPD=M@LCLM=D@R15A=M0;JMP(boolean_cmd_helper_JLT)@SPA=MA=A-1D=MA=A-1D=M-D@BOOL_JLTD;JLT@SPA=MA=A-1A=A-1M=0@END_BOOL_JLT0;JMP(BOOL_JLT)@SPA=MA=A-1A=A-1M=-1(END_BOOL_JLT)@SPM=M-1@R15A=M0;JMP(boolean_cmd_helper_JGT)@SPA=MA=A-1D=MA=A-1D=M-D@BOOL_JGTD;JGT@SPA=MA=A-1A=A-1M=0@END_BOOL_JGT0;JMP(BOOL_JGT)@SPA=MA=A-1A=A-1M=-1(END_BOOL_JGT)@SPM=M-1@R15A=M0;JMP(boolean_cmd_helper_JEQ)@SPA=MA=A-1D=MA=A-1D=M-D@BOOL_JEQD;JEQ@SPA=MA=A-1A=A-1M=0@END_BOOL_JEQ0;JMP(BOOL_JEQ)@SPA=MA=A-1A=A-1M=-1(END_BOOL_JEQ)@SPM=M-1@R15A=M0;JMP// function SimpleFunction.test 2(SimpleFunction.test)// Push 0@0D=A@SPA=MM=D@SPM=M+1// Push 0@0D=A@SPA=MM=D@SPM=M+1// push local 0@LCLA=MD=M@SPA=MM=D@SPM=M+1// push local 1@LCLA=MD=A@1A=D+AD=M@SPA=MM=D@SPM=M+1// add// add@SPA=MA=A-1D=MA=A-1M=D+M@SPM=M-1// not@SPA=MA=A-1M=!M// push argument 0@ARGA=MD=M@SPA=MM=D@SPM=M+1// add// add@SPA=MA=A-1D=MA=A-1M=D+M@SPM=M-1// push argument 1@ARGA=MD=A@1A=D+AD=M@SPA=MM=D@SPM=M+1// sub// sub@SPA=MA=A-1D=MA=A-1M=M-D@SPM=M-1// return// goto restore_stack_and_return@restore_stack_and_return0;JMP