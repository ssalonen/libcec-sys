#include <cecc.h>

int main()
{
    _Static_assert(CEC_LIB_VERSION_MAJOR == 6, "LIBCEC != v6.x.y");
    return (int)libcec_initialise;    
}
