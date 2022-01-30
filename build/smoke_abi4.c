#include <cecc.h>

int main()
{
    _Static_assert(CEC_LIB_VERSION_MAJOR == 4, "LIBCEC != v4.x.y");
    return (int)libcec_initialise;    
}
