

#include <ctype.h>
#include <gc.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef void *(*ToStringFunc)(void *this);

void *cls_String_cons_internal_alloc(char *str);
void *cls_String_cons_internal_format(char *fmt, ...);

// AnyRef
struct AnyRef {
  void **vtable;
};

void cls_AnyRef_init(void *this);
void *cls_AnyRef_cons_args_();
void *cls_AnyRef_method_ToString_args_(void *this);

extern void *cls_AnyRef_vtable_data[];

void cls_AnyRef_init(void *this) {}

void *cls_AnyRef_cons_args_() {
  struct AnyRef *result = GC_malloc(sizeof(struct AnyRef));
  result->vtable = cls_AnyRef_vtable_data;
  cls_AnyRef_init(result);
  return result;
}

void *cls_AnyRef_method_ToString_args_(void *this) {
  return cls_String_cons_internal_alloc("AnyRef");
}

// Array
struct Array {
  struct AnyRef anyref;
  int32_t len;
  void **array;
};

void cls_Array_init(void *this);
void *cls_Array_cons_args_Integer(int32_t len);
void cls_Array_method_Set_args_Integer_AnyRef(void *this, int32_t idx,
                                              void *item);
void *cls_Array_method_Get_args_Integer(void *this, int32_t idx);
int32_t cls_Array_method_Length_args_(void *this);

extern void *cls_Array_vtable_data[];

void cls_Array_init(void *this) { ((struct Array *)this)->len = 0; }
void *cls_Array_cons_args_Integer(int32_t len) {
  struct Array *result = GC_malloc(sizeof(struct Array));
  result->anyref.vtable = cls_Array_vtable_data;
  cls_Array_init(result);
  result->len = len;
  result->array = GC_malloc(sizeof(void *) * len);
  for (int32_t i = 0; i < len; i++) {
    result->array[i] = cls_AnyRef_cons_args_();
  }
  return result;
}
void cls_Array_method_Set_args_Integer_AnyRef(void *this, int32_t idx,
                                              void *item) {
  ((struct Array *)this)->array[idx] = item;
}
void *cls_Array_method_Get_args_Integer(void *this, int32_t idx) {
  return ((struct Array *)this)->array[idx];
}
int32_t cls_Array_method_Length_args_(void *this) {
  return ((struct Array *)this)->len;
}

// String
struct String {
  struct AnyRef anyref;
  int32_t len;
  char *data;
};

void cls_String_init(void *this);
void *cls_String_cons_args_();
void *cls_String_method_ToString_args_(void *this);
bool cls_String_method_IsInteger_args_(void *this);
int32_t cls_String_method_ToInteger_args_(void *this);
bool cls_String_method_IsReal_args_(void *this);
float cls_String_method_ToReal_args_(void *this);
bool cls_String_method_IsBoolean_args_(void *this);
bool cls_String_method_ToBoolean_args_(void *this);
bool cls_String_method_Equal_args_String(void *this, void *other);
void *cls_String_method_Concat_args_String(void *this, void *other);

extern void *cls_String_vtable_data[];

void cls_String_init(void *this) { ((struct String *)this)->len = 0; }

void *cls_String_cons_args_() {
  struct String *result = GC_malloc(sizeof(struct String));
  result->anyref.vtable = cls_String_vtable_data;
  cls_String_init(result);
  result->len = 0;
  result->data = GC_malloc(sizeof(char) * 1);
  result->data[0] = '\0';
  return result;
}
void *cls_String_method_ToString_args_(void *this) { return this; }

void *cls_String_cons_internal_alloc(char *str) {
  struct String *result = GC_malloc(sizeof(struct String));
  result->anyref.vtable = cls_String_vtable_data;
  cls_String_init(result);
  result->len = strlen(str);
  result->data = GC_malloc(sizeof(char) * (result->len + 1));
  strcpy(result->data, str);
  return result;
}
void *cls_String_cons_internal_take(char *str) {
  struct String *result = GC_malloc(sizeof(struct String));
  result->anyref.vtable = cls_String_vtable_data;
  cls_String_init(result);
  result->len = strlen(str);
  result->data = str;
  return result;
}

void *cls_String_cons_internal_format(char *fmt, ...) {
  struct String *result = GC_malloc(sizeof(struct String));
  result->anyref.vtable = cls_String_vtable_data;
  cls_String_init(result);

  va_list args;
  va_start(args, fmt);
  int size = vsnprintf(NULL, 0, fmt, args);
  va_end(args);

  result->len = size;
  result->data = GC_malloc(sizeof(char) * (size + 1));

  va_start(args, fmt);
  vsnprintf(result->data, size + 1, fmt, args);
  va_end(args);
  return result;
}

bool cls_String_method_IsInteger_args_(void *this) {
  char *str = ((struct String *)this)->data;
  if (str == NULL || *str == '\0')
    return false;
  while (*str) {
    if (!isdigit(*str))
      return false;
    str++;
  }
  return true;
}
int32_t cls_String_method_ToInteger_args_(void *this) {
  char *str = ((struct String *)this)->data;
  return atoi(str);
}
bool cls_String_method_IsReal_args_(void *this) {
  char *str = ((struct String *)this)->data;

  if (str == NULL || *str == '\0' || isspace(*str))
    return false;

  char *endptr;
  strtof(str, &endptr);

  return *endptr == '\0';
}
float cls_String_method_ToReal_args_(void *this) {
  char *str = ((struct String *)this)->data;
  return atof(str);
}
bool cls_String_method_IsBoolean_args_(void *this) {
  char *str = ((struct String *)this)->data;
  return strcmp(str, "true") == 0 || strcmp(str, "false") == 0;
}
bool cls_String_method_ToBoolean_args_(void *this) {
  char *str = ((struct String *)this)->data;
  return strcmp(str, "true") == 0 ? true : false;
}

bool cls_String_method_Equal_args_String(void *this, void *other) {
  char *str1 = ((struct String *)this)->data;
  char *str2 = ((struct String *)other)->data;
  return strcmp(str1, str2) == 0;
}

void *cls_String_method_Concat_args_String(void *this, void *other) {
  struct String *str1 = ((struct String *)this);
  struct String *str2 = ((struct String *)other);
  return cls_String_cons_internal_format("%s%s", str1->data, str2->data);
}

void *cls_Main_cons_args_Array(void *array);

int main(int argc, char **argv) {
  GC_init();
  void *array = cls_Array_cons_args_Integer(argc - 1);
  for (int i = 1; i < argc; i++) {
    cls_Array_method_Set_args_Integer_AnyRef(
        array, i - 1, cls_String_cons_internal_alloc(argv[i]));
  }
  struct AnyRef *main = (struct AnyRef *)cls_Main_cons_args_Array(array);
  struct String *res = (struct String *)((ToStringFunc)main->vtable[0])(main);

  printf("%s\n", res->data);
  GC_gcollect();
  return 0;
}