# searust

To try this out:

Set up a sysroot directory:

    $ mkdir sysroot
    $ cd sysroot
    $ export SYSROOT=$(pwd)

Run the kernel and core build:

    $ cd sel4-build
    $ ./build.sh

(This will install the kernel and libraries to the sysroot)

Run the userspace build:

    $ cd corerust
    $ make install

(This will install the rootserver to the sysroot)

Try running the machine:

    $ ./test.sh

Note that this currently causes a fault, but you should at least see seL4
itself booting up and trying to launch the rust program.
