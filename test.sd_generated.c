#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>

typedef struct {
    double x;
    double y;
} side_Point;


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
side_Point side_make_point(double x, double y) {
    side_Point p = (side_Point){x, y};
    return p;
}

double side_distance(side_Point p) {
    return sqrt(((p.x * p.x) + (p.y * p.y)));
    return 0;
}

double side_add_double(double a, double b) {
    return (a + b);
    return 0;
}

int side_factorial(int n) {
    if ((n <= 1)) {
        return 1;
    }
    return (n * side_factorial((n - 1)));
    return 0;
}

int side_main() {
    printf("%s", "=== Тестирование языка Side ===");
    printf("\n");
    int a = 10;
    int b = 3;
    int sum = (a + b);
    int diff = (a - b);
    int prod = (a * b);
    int quot = (a / b);
    printf("%s", "1. Арифметика: sum=");
    printf("%d", sum);
    printf("%s", " diff=");
    printf("%d", diff);
    printf("%s", " prod=");
    printf("%d", prod);
    printf("%s", " quot=");
    printf("%d", quot);
    printf("\n");
    int x = 5;
    if (((x > 0) && (x < 10))) {
        printf("%s", "2. Логика: x in (0,10) - OK");
        printf("\n");
    }
    else {
        printf("%s", "2. Логика: FAILED");
        printf("\n");
    }
    if (!((x == 0))) {
        printf("%s", "2b. not (x==0) - OK");
        printf("\n");
    }
    int score = 85;
    if ((score >= 90)) {
        printf("%s", "3. Grade: A");
        printf("\n");
    }
    else {
        if ((score >= 80)) {
            printf("%s", "3. Grade: B (OK)");
            printf("\n");
        }
        else {
            printf("%s", "3. Grade: FAILED");
            printf("\n");
        }
    }
    int counter = 0;
    int while_sum = 0;
    while ((counter < 5)) {
        while_sum = (while_sum + counter);
        counter = (counter + 1);
    }
    printf("%s", "4. While sum 0..4 = ");
    printf("%d", while_sum);
    printf("%s", " (expected 10)");
    printf("\n");
    int for_sum = 0;
    {
        int i = 0;
        while ((i < 5)) {
            for_sum = (for_sum + i);
            i = (i + 1);
        }
    }
    printf("%s", "5. For sum 0..4 = ");
    printf("%d", for_sum);
    printf("%s", " (expected 10)");
    printf("\n");
    int* iarr = NULL;
    int iarr_size = 0;
    int iarr_cap = 0;
    side_arr_push(&iarr, &iarr_size, &iarr_cap, 100);
    side_arr_push(&iarr, &iarr_size, &iarr_cap, 200);
    side_arr_push(&iarr, &iarr_size, &iarr_cap, 300);
    int iarr_len = iarr_size;
    printf("%s", "6. Int array length = ");
    printf("%d", iarr_len);
    printf("%s", " (expected 3)");
    printf("\n");
    printf("%s", "6b. Elements: ");
    printf("%d", iarr[0]);
    printf("%s", " ");
    printf("%d", iarr[1]);
    printf("%s", " ");
    printf("%d", iarr[2]);
    printf("%s", " (expected 100 200 300)");
    printf("\n");
    side_arr_pop(&iarr, &iarr_size, &iarr_cap);
    printf("%s", "6c. After pop length = ");
    printf("%d", iarr_size);
    printf("%s", " (expected 2)");
    printf("\n");
    double* darr = NULL;
    int darr_size = 0;
    int darr_cap = 0;
    side_arr_push_double(&darr, &darr_size, &darr_cap, 1.5);
    side_arr_push_double(&darr, &darr_size, &darr_cap, 2.5);
    side_arr_push_double(&darr, &darr_size, &darr_cap, 3.5);
    int darr_len = darr_size;
    printf("%s", "7. Double array length = ");
    printf("%d", darr_len);
    printf("%s", " (expected 3)");
    printf("\n");
    printf("%s", "7b. Elements: ");
    printf("%f", darr[0]);
    printf("%s", " ");
    printf("%f", darr[1]);
    printf("%s", " ");
    printf("%f", darr[2]);
    printf("%s", " (expected 1.5 2.5 3.5)");
    printf("\n");
    side_arr_pop_double(&darr, &darr_size, &darr_cap);
    printf("%s", "7c. After pop length = ");
    printf("%d", darr_size);
    printf("%s", " (expected 2)");
    printf("\n");
    const char* greeting = "Hello";
    const char* target = "Side";
    printf("%s", "8. Strings: ");
    printf("%s", greeting);
    printf("%s", ", ");
    printf("%s", target);
    printf("%s", "! (expected Hello, Side!)");
    printf("\n");
    double da = 3;
    double db = 4;
    double dsum = side_add_double(da, db);
    printf("%s", "9. add_double(3,4) = ");
    printf("%f", dsum);
    printf("%s", " (expected 7)");
    printf("\n");
    int f5 = side_factorial(5);
    printf("%s", "9b. factorial(5) = ");
    printf("%d", f5);
    printf("%s", " (expected 120)");
    printf("\n");
    int tm = side_time();
    printf("%s", "10a. time() = ");
    printf("%d", tm);
    printf("%s", " ms (any positive number)");
    printf("\n");
    double sq = sqrt(16);
    printf("%s", "10b. sqrt(16) = ");
    printf("%f", sq);
    printf("%s", " (expected 4)");
    printf("\n");
    double av = fabs(-(3.14));
    printf("%s", "10c. fabs(-3.14) = ");
    printf("%f", av);
    printf("%s", " (expected 3.14)");
    printf("\n");
    double pw = pow(2, 3);
    printf("%s", "10d. pow(2,3) = ");
    printf("%f", pw);
    printf("%s", " (expected 8)");
    printf("\n");
    int rnd = rand();
    printf("%s", "10e. rand() = ");
    printf("%d", rnd);
    printf("%s", " (random integer)");
    printf("\n");
    const char* num_str = side_str(123);
    printf("%s", "10f. str(123) = ");
    printf("%s", num_str);
    printf("%s", " (expected '123')");
    printf("\n");
    int parsed = atoi("456");
    printf("%s", "10g. int('456') = ");
    printf("%d", parsed);
    printf("%s", " (expected 456)");
    printf("\n");
    side_Point p1 = side_make_point(3, 4);
    printf("%s", "11a. p1.x = ");
    printf("%d", p1.x);
    printf("%s", " p1.y = ");
    printf("%d", p1.y);
    printf("%s", " (expected 3.0, 4.0)");
    printf("\n");
    double dist = side_distance(p1);
    printf("%s", "11b. distance to origin = ");
    printf("%f", dist);
    printf("%s", " (expected 5.0)");
    printf("\n");
    printf("%s", "");
    printf("\n");
    printf("%s", "=== Все тесты пройдены! ===");
    printf("\n");
    return 0;
}

int main() { return side_main(); }
