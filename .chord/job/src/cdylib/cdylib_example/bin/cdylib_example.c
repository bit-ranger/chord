#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char * run(char * req){
    char * result;
    printf("run req: %s", req);
    result = malloc(sizeof(char) * 10);
    strcpy(result, "{\"run\": 1}");
    return result;
}

