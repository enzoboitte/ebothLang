#include<stdint.h>
#include<unistd.h>
#include<sys/mman.h>
#include<errno.h>

void dump_i(int64_t x)
{
    char buf[32];
    size_t pos = sizeof(buf);

    int sign = 0;
    if (x < 0) 
    {
        sign = 1;
        x = -x;
    }

    do {
        buf[--pos] = x % 10 + '0';
        x /= 10;
    } while (x);

    if (sign)
        buf[--pos] = '-';

    write(1, &buf[pos], sizeof(buf) - pos);
}


void dump_str(const char* s)
{
    size_t len = 0;
    while (s[len]) len++;
    write(1, s, len);
}

int f_bIsValidPtr(void* p)
{
    size_t l_uPageSize = sysconf(_SC_PAGESIZE);
    void* l_pBase = (void*)(((size_t)p / l_uPageSize) * l_uPageSize);
    int l_iRet = msync(l_pBase, l_uPageSize, MS_ASYNC);
    return l_iRet == 0 || errno != ENOMEM;
}

int f_bIsValidStr(const char* s)
{
    if (!f_bIsValidPtr((void*)s)) return 0;
    
    for (int i = 0; i < 4096; i++) {
        if (!f_bIsValidPtr((void*)(s + i))) return 0;
        if (s[i] == '\0') return 1;
        if (s[i] < 32 || s[i] > 126) return 0;
    }
    return 0;
}

void dump(void* p)
{
    if (f_bIsValidStr((const char*)p)) {
        dump_str((const char*)p);
    } else {
        dump_i((uintptr_t)p);
    }
}

int main(void)
{
    dump_i(-12345678910);
    //dump_str("Hello, World!");
    
    //dump((void*)100000);
    //dump((void*)"Goodbye, World!");
    
    return 0;
}
