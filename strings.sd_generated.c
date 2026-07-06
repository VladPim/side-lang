#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>


void side_arr_push(int** arr, int* size, int* cap, int value) {
    if (*size >= *cap) { *cap = (*cap == 0) ? 2 : (*cap) * 2; *arr = realloc(*arr, (*cap) * sizeof(int)); }
    (*arr)[*size] = value; (*size)++;
}
int side_arr_pop(int** arr, int* size, int* cap) {
    if (*size <= 0) return -1; int val = (*arr)[*size - 1]; (*size)--; return val;
}
int* side_arr_create(int n, int* vals) {
    int* arr = malloc(n * sizeof(int)); for (int i = 0; i < n; i++) arr[i] = vals[i]; return arr;
}
void side_arr_push_double(double** arr, int* size, int* cap, double value) {
    if (*size >= *cap) { *cap = (*cap == 0) ? 2 : (*cap) * 2; *arr = realloc(*arr, (*cap) * sizeof(double)); }
    (*arr)[*size] = value; (*size)++;
}
double side_arr_pop_double(double** arr, int* size, int* cap) {
    if (*size <= 0) return -1.0; double val = (*arr)[*size - 1]; (*size)--; return val;
}
double* side_arr_create_double(int n, double* vals) {
    double* arr = malloc(n * sizeof(double)); for (int i = 0; i < n; i++) arr[i] = vals[i]; return arr;
}
int side_input(const char* prompt) { int val; printf("%s", prompt); scanf("%d", &val); return val; }
int side_time() { return (int)(clock() * 1000 / CLOCKS_PER_SEC); }
char* side_str(int n) { static char buf[20]; sprintf(buf, "%d", n); return buf; }
char* side_str_double(double n) { static char buf[30]; sprintf(buf, "%f", n); return buf; }
char* side_str_concat(const char* a, const char* b) {
    char* result = malloc(strlen(a) + strlen(b) + 1);
    strcpy(result, a);
    strcat(result, b);
    return result;
}
int side_main() {
    const char* a = "Hello";
    const char* b = "World";
    const char* c = side_str_concat(side_str_concat(a, " "), b);
    printf("%s", "Concatenated: ");
    printf("%s", c);
    printf("\n");
    if ((strcmp(c, "Hello World") == 0)) {
        printf("%s", "Equality test: OK");
        printf("\n");
    }
    else {
        printf("%s", "Equality test: FAILED");
        printf("\n");
    }
    return 0;
}

int main() { return side_main(); }
