#include <stdint.h>
#include <cecc.h>

int main()
{
    _Static_assert(CEC_LIB_VERSION_MAJOR == 4, 
        "libcec major version is " CEC_LIB_VERSION_MAJOR_STR ", not as expected (4)");
    return (intptr_t)libcec_initialise;
}
