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

#include "los_printf.h"
#include "los_task_pri.h"
#include "los_typedef.h"

#ifdef LOSCFG_SHELL
extern VOID   OsShellCmdSystemInfoGet(VOID);
extern UINT32 OsShellCmdFree(INT32 argc, const CHAR *argv[]);
extern UINT32 OsShellCmdUname(INT32 argc, const CHAR *argv[]);
extern UINT32 OsShellCmdDumpPmm(VOID);
#endif

#define HIDUMPER_KERNEL_FAULT_ADDR  0x1
#define HIDUMPER_KERNEL_FAULT_VALUE 0x2

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
