/* Rosemary glue for vr_device.c and vr_xr.c.
 *
 * Under Darling the shim reaches host (Linux) libraries through the table
 * Darling's loader publishes as `_elfcalls`. vr_device.c looks that table up
 * with dlsym(RTLD_DEFAULT, "_elfcalls"), so under Rosemary this file publishes
 * a table of the same shape, backed by Rosemary's rosemary_host_* exports (the
 * host loader's own dlopen/dlsym; see rovarch-core
 * crates/darwin/src/rosemary/hostland.rs).
 *
 * The sources of vr_device.c and vr_xr.c are used unchanged.
 */

typedef struct {
    void *(*open)(const char *, int);
    int (*close)(void *);
    void *(*symbol)(void *, const char *);
} RosemaryElfCalls;

/* Weak: a Rosemary without the bridge leaves them null, and VR stays off
 * with a log line instead of a crash. */
extern void *rosemary_host_dlopen(const char *, int) __attribute__((weak_import));
extern void *rosemary_host_dlsym(void *, const char *) __attribute__((weak_import));

static void *glue_open(const char *library, int mode) {
    if (!rosemary_host_dlopen) return 0;
    return rosemary_host_dlopen(library, mode);
}

/* Host libraries stay loaded: pointers into them are held for the process. */
static int glue_close(void *library) {
    (void)library;
    return 0;
}

static void *glue_symbol(void *library, const char *name) {
    return rosemary_host_dlsym ? rosemary_host_dlsym(library, name) : 0;
}

static RosemaryElfCalls glue_table = {glue_open, glue_close, glue_symbol};

/* The pointer vr_device.c reads through dlsym(RTLD_DEFAULT, "_elfcalls"). */
RosemaryElfCalls *_elfcalls = &glue_table;
