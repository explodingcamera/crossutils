#include "cosmopolitan.h"

int pthread_sigmask(int how, const sigset_t *set, sigset_t *oldset) {
    printf("sigmask! %i\n", how);
    return 0;
}
