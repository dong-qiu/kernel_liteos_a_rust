/*
 * Copyright (c) 2024 Huawei Device Co., Ltd. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice, this list of
 *    conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice, this list
 *    of conditions and the following disclaimer in the documentation and/or other materials
 *    provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its contributors may be used
 *    to endorse or promote products derived from this software without specific prior written
 *    permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS;
 * OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
 * WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR
 * OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
 * ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#include "los_cpup.h"
#include "los_memory.h"
#include "los_printf.h"
#include "los_process_pri.h"
#include "los_task_pri.h"
#include "los_typedef.h"
#ifdef LOSCFG_FS_VFS
#include "fs/file.h"
#endif
#ifdef LOSCFG_BLACKBOX
#include "los_blackbox.h"
#endif

#ifdef LOSCFG_SHELL
extern VOID   OsShellCmdSystemInfoGet(VOID);
extern UINT32 OsShellCmdFree(INT32 argc, const CHAR *argv[]);
extern UINT32 OsShellCmdUname(INT32 argc, const CHAR *argv[]);
extern UINT32 OsShellCmdDumpPmm(VOID);
#endif

#define HIDUMPER_KERNEL_FAULT_ADDR  0x1
#define HIDUMPER_KERNEL_FAULT_VALUE 0x2

typedef struct {
    const CHAR *name;
    UINT32 pid;
    UINT8 unused;
} HidumperProcessInfo;

void HidumperPrintk(const char *msg)
{
    if (msg == NULL) {
        return;
    }
    PRINTK("%s", msg);
}

int HidumperShellCmdUname(void)
{
#ifdef LOSCFG_SHELL
    const char *argv[1] = {"-a"};
    (VOID)OsShellCmdUname((INT32)(sizeof(argv) / sizeof(argv[0])), &argv[0]);
    return 0;
#else
    return -1;
#endif
}

int HidumperShellCmdSystemInfo(void)
{
#ifdef LOSCFG_SHELL
    (VOID)OsShellCmdSystemInfoGet();
    return 0;
#else
    return -1;
#endif
}

int HidumperShellCmdFree(void)
{
#ifdef LOSCFG_SHELL
    const char *argv[1] = {"-k"};
    (VOID)OsShellCmdFree((INT32)(sizeof(argv) / sizeof(argv[0])), &argv[0]);
    return 0;
#else
    return -1;
#endif
}

int HidumperShellCmdDumpPmm(void)
{
#ifdef LOSCFG_SHELL
    (VOID)OsShellCmdDumpPmm();
    return 0;
#else
    return -1;
#endif
}

int HidumperShellCmdTaskInfo(void)
{
#ifdef LOSCFG_SHELL
    (VOID)OsShellCmdTskInfoGet(OS_ALL_TASK_MASK, NULL, OS_PROCESS_INFO_ALL);
    return 0;
#else
    return -1;
#endif
}

int HidumperInjectKernelCrash(void)
{
#ifdef LOSCFG_DEBUG_VERSION
    *((INT32 *)HIDUMPER_KERNEL_FAULT_ADDR) = HIDUMPER_KERNEL_FAULT_VALUE;
    return 0;
#else
    return -1;
#endif
}

const char *HidumperGetKernelFaultLogPath(void)
{
#ifdef LOSCFG_BLACKBOX
    return KERNEL_FAULT_LOG_PATH;
#else
    return NULL;
#endif
}

const char *HidumperGetUserFaultLogPath(void)
{
#ifdef LOSCFG_BLACKBOX
    return USER_FAULT_LOG_PATH;
#else
    return NULL;
#endif
}

int HidumperOpenReadOnly(const char *path)
{
#ifdef LOSCFG_FS_VFS
    if (path == NULL) {
        return -1;
    }
    return open(path, O_RDONLY);
#else
    (VOID)path;
    return -1;
#endif
}

int HidumperIsVfsEnabled(void)
{
#ifdef LOSCFG_FS_VFS
    return 1;
#else
    return 0;
#endif
}

void HidumperLogOpenFailed(const char *path)
{
    if (path == NULL) {
        PRINT_ERR("filePath is NULL!\n");
        return;
    }
    PRINT_ERR("Open [%s] failed or there's no fault log!\n", path);
}

void HidumperLogVfsUnsupported(void)
{
    PRINT_ERR("LOSCFG_FS_VFS isn't defined!\n");
}

int HidumperClose(int fd)
{
#ifdef LOSCFG_FS_VFS
    return close(fd);
#else
    (VOID)fd;
    return -1;
#endif
}

int HidumperRead(int fd, char *buf, unsigned int len)
{
#ifdef LOSCFG_FS_VFS
    return read(fd, buf, len);
#else
    (VOID)fd;
    (VOID)buf;
    (VOID)len;
    return -1;
#endif
}

UINT32 HidumperGetProcessMaxNum(void)
{
    return g_processMaxNum;
}

INT32 HidumperGetProcessInfo(UINT32 pid, HidumperProcessInfo *out)
{
    if ((out == NULL) || (pid >= g_processMaxNum)) {
        return -1;
    }

    LosProcessCB *pcb = g_processCBArray + pid;
    out->name = pcb->processName;
    out->pid = pcb->processID;
    out->unused = OsProcessIsUnused(pcb) ? 1 : 0;
    return 0;
}

UINT32 HidumperGetAllProcessCpuUsage(UINT16 mode, CPUP_INFO_S *cpupInfo, UINT32 len)
{
#ifdef LOSCFG_KERNEL_CPUP
    return LOS_GetAllProcessCpuUsage(mode, cpupInfo, len);
#else
    (VOID)mode;
    (VOID)cpupInfo;
    (VOID)len;
    return (UINT32)-1;
#endif
}

VOID *HidumperMemAlloc(UINT32 size)
{
    return LOS_MemAlloc(m_aucSysMem1, size);
}

VOID HidumperMemFree(VOID *ptr)
{
    if (ptr != NULL) {
        (VOID)LOS_MemFree(m_aucSysMem1, ptr);
    }
}

VOID HidumperPrintCpuUsageHeader(void)
{
    PRINTK("%-32s PID CPUUSE CPUUSE10S CPUUSE1S\n", "PName");
}

VOID HidumperPrintCpuUsageLine(const CHAR *name, UINT32 pid, UINT32 all, UINT32 ten, UINT32 one)
{
    PRINTK("%-32s %u %5u.%1u%8u.%1u%7u.%-1u\n",
        name, pid,
        all / LOS_CPUP_PRECISION_MULT, all % LOS_CPUP_PRECISION_MULT,
        ten / LOS_CPUP_PRECISION_MULT, ten % LOS_CPUP_PRECISION_MULT,
        one / LOS_CPUP_PRECISION_MULT, one % LOS_CPUP_PRECISION_MULT);
}
