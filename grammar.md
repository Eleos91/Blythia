||||
|---|---|---|
|START|:=|STATEMENT|
|STATEMENT|:=|WHILE \| IF \| DECLARATION \| ASSIGNEMT \| EXPRESSION \| FUNC_DEF|
|WHILE|:=|while EXPRESSION : NEWLINE_INDENT STATEMENT [ NEWLINE STATEMENT ]*|
|IF|:=|if EXPRESSION : NEWLINE_INDENT STATEMENT [ NEWLINE STATEMENT ]* [ ELIF ] [ ELSE ]
|ELIF|:=|elif EXPRESSION : NEWLINE_INDENT STATEMENT [ NEWLINE STATEMENT ]*|
|ELSE|:=|else : NEWLINE_INDENT STATEMENT [ NEWLINE STATEMENT ]*|
|DECLARATION|:=|var VAR_NAME : TYPE [ = EXPRESSION ]|
|ASSIGNEMT|:=|VAR_NAME = EXPRESSION|
|EXPRESSION|:=|OPERATION \| VAR_NAME \| FUNC_CALL \| BUILTIN|
|FUNC_CALL|:=|FUNC_NAME([ ARGUMENTS ])|
|ARGUMENTS|:=|VAR_NAME [ , VAR_NAME ]*|
|BUILTIN|:=|print_int(VAR_NAME)|
|FUNC_DEF|:=|def FUNC_NAME([ PARAMETERS ]) -> TYPE : NEWLINE_INDENT STATEMENT [ NEWLINE STATEMENT ]*|
|PARAMETERS|:=|VAR_NAME : TYPE [ , VAR_NAME : TYPE ]*||
