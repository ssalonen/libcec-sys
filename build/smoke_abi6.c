#include <stdint.h>
#include <cecc.h>

int main()
{
    _Static_assert(CEC_LIB_VERSION_MAJOR == 6, 
        "libcec major version is " CEC_LIB_VERSION_MAJOR_STR ", not as expected (6)");
    return (intptr_t)libcec_initialise;
}
