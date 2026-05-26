
#include <ctype.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum ship_arg_type { ARG_INT = 3, ARG_REAL = 4, ARG_BOOL = 5 };

union ship_arg_value {
  int32_t int_value;
  bool bool_value;
  float real_value;
};

struct ship_arg {
  int32_t type;
  union ship_arg_value value;
};

typedef void *(*ship_cons_func)(void);
typedef void *(*ship_cons_func1)(union ship_arg_value arg1);
typedef void *(*ship_cons_func2)(union ship_arg_value arg1,
                                 union ship_arg_value arg2);
typedef void *(*ship_cons_func3)(union ship_arg_value arg1,
                                 union ship_arg_value arg2,
                                 union ship_arg_value arg3);

struct ship_cons_meta {
  int32_t arg_count;
  int32_t *args;
  ship_cons_func func;
};

struct ship_cls_meta {
  char *name;
  int32_t cons_count;
  struct ship_cons_meta *constructors;
};

extern struct ship_cls_meta cls_registry[];
extern int32_t cls_registry_size;

static bool is_str_numeric(char *str) {
  if (str == NULL || *str == '\0')
    return false;
  while (*str) {
    if (!isdigit(*str))
      return false;
    str++;
  }
  return true;
}

static bool is_str_float(char *str) {
  if (str == NULL || *str == '\0' || isspace(*str))
    return false;

  char *endptr;
  strtof(str, &endptr);

  return *endptr == '\0';
}

static void *call_cons(struct ship_cons_meta *cons, struct ship_arg *args) {
  switch (cons->arg_count) {
  case 0:
    return cons->func();
  case 1:
    return ((ship_cons_func1)cons->func)(args[0].value);
  case 2:
    return ((ship_cons_func2)cons->func)(args[0].value, args[1].value);

  case 3:
    return ((ship_cons_func3)cons->func)(args[0].value, args[1].value,
                                         args[2].value);
  }
}

int main(int argc, char **argv) {

  if (argc < 2) {
    fprintf(stderr, "Usage: %s <class name> [constructor args]", argv[0]);
    return 1;
  }

  char *cls_name = argv[1];

  struct ship_arg *args = malloc(sizeof(struct ship_arg) * (argc - 2));
  for (int32_t i = 2; i < argc - 2; i++) {
    char *arg = argv[i];

    if (strcmp(arg, "true") == 0) {
      args[i - 2].type = ARG_BOOL;
      args[i - 2].value.bool_value = true;
    } else if (strcmp(arg, "false") == 0) {
      args[i - 2].type = ARG_BOOL;
      args[i - 2].value.bool_value = false;
    } else if (is_str_numeric(arg)) {
      args[i - 2].type = ARG_INT;
      args[i - 2].value.int_value = atoi(arg);
    } else if (is_str_float(arg)) {
      args[i - 2].type = ARG_REAL;
      args[i - 2].value.real_value = atof(arg);
    } else {
      fprintf(stderr, "Couldn't parse argument '%s'", arg);
      free(args);
      return 1;
    }
  }

  for (int32_t cls_idx = 0; cls_idx < cls_registry_size; cls_idx++) {
    struct ship_cls_meta *cls = &cls_registry[cls_idx];
    if (strcmp(cls_name, cls->name) == 0) {
      for (int32_t cons_idx = 0; cons_idx < cls->cons_count; cons_idx++) {
        struct ship_cons_meta *cons = &cls->constructors[cons_idx];
        if (cons->arg_count == argc - 2) {
          for (int32_t arg_idx = 0; arg_idx < cons->arg_count; arg_idx++) {
            if (cons->args[arg_idx] != args[arg_idx].type)
              break;
          }
          call_cons(cons, args);
        }
      }
    }
  }
}