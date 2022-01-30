#include <cecc.h>

int main()
{
    _Static_assert(CEC_LIB_VERSION_MAJOR == 5, "LIBCEC != v5.x.y");
    return (int)libcec_initialise;    
}
