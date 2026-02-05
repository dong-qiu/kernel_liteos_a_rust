/*
 * Copyright (c) 2021-2021 Huawei Device Co., Ltd. All rights reserved.
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

/* ------------ includes ------------ */
#include "los_blackbox_common.h"
#ifdef LOSCFG_LIB_LIBC
#include "stdlib.h"
#include "unistd.h"
#endif
#ifdef LOSCFG_FS_VFS
#include "fs/fs.h"
#include "fs/mount.h"
#endif
#include "securec.h"
#include "los_memory.h"

/* ------------ local macroes ------------ */
#ifdef LOSCFG_FS_VFS
#define BBOX_DIR_MODE 0750
#define BBOX_FILE_MODE 0640
#endif

/* ------------ local prototypes ------------ */
/* ------------ local function declarations ------------ */
/* ------------ global function declarations ------------ */
/* ------------ local variables ------------ */
static bool g_isLogPartReady = FALSE;

void BlackboxLogErrSimple(const char *msg)
{
    if (msg == NULL) {
        BBOX_PRINT_ERR("msg is NULL!\n");
        return;
    }
    BBOX_PRINT_ERR("%s", msg);
}

void BlackboxLogInfoSimple(const char *msg)
{
    if (msg == NULL) {
        BBOX_PRINT_INFO("msg is NULL!\n");
        return;
    }
    BBOX_PRINT_INFO("%s", msg);
}

void BlackboxLogErrModule(const char *module, const char *msg)
{
    if (module == NULL || msg == NULL) {
        BBOX_PRINT_ERR("module: %p, msg: %p!\n", module, msg);
        return;
    }
    BBOX_PRINT_ERR("[%s] %s", module, msg);
}

void BlackboxLogInfoModule(const char *module, const char *msg)
{
    if (module == NULL || msg == NULL) {
        BBOX_PRINT_INFO("module: %p, msg: %p!\n", module, msg);
        return;
    }
    BBOX_PRINT_INFO("[%s] %s", module, msg);
}

void BlackboxLogInfoModuleEvent(const char *module, const char *event, const char *msg)
{
    if (module == NULL || event == NULL || msg == NULL) {
        BBOX_PRINT_INFO("module: %p, event: %p, msg: %p!\n", module, event, msg);
        return;
    }
    BBOX_PRINT_INFO("[%s] %s [%s]\n", module, msg, event);
}

void BlackboxLogErrPathFailed(const char *prefix, const char *path)
{
    if (prefix == NULL || path == NULL) {
        BBOX_PRINT_ERR("prefix: %p, path: %p!\n", prefix, path);
        return;
    }
    BBOX_PRINT_ERR("%s [%s] failed!\n", prefix, path);
}

void BlackboxLogInvalidWriteArgs(const char *filePath, const void *buf, size_t bufSize)
{
    BBOX_PRINT_ERR("filePath: %p, buf: %p, bufSize: %lu!\n", filePath, buf, bufSize);
}

void BlackboxLogLogPartNotReady(void)
{
    BBOX_PRINT_ERR("log path [%s] isn't ready to be written!\n", LOSCFG_BLACKBOX_LOG_ROOT_PATH);
}

void BlackboxLogOpenFailed(const char *filePath, int fd)
{
    if (filePath == NULL) {
        BBOX_PRINT_ERR("filePath is NULL!\n");
        return;
    }
    BBOX_PRINT_ERR("Create file [%s] failed, fd: %d!\n", filePath, fd);
}

void BlackboxLogWriteFailed(const char *filePath)
{
    if (filePath == NULL) {
        BBOX_PRINT_ERR("filePath is NULL!\n");
        return;
    }
    BBOX_PRINT_ERR("Failed to write file [%s]!\n", filePath);
}

void BlackboxLogBufferNotEnough(void)
{
    BBOX_PRINT_ERR("buf is not enough or snprintf_s failed!\n");
}

/* ------------ function definitions ------------ */
int FullWriteFile(const char *filePath, const char *buf, size_t bufSize, int isAppend)
{
#ifdef BLACKBOX_USE_RUST
    extern int BlackboxFullWriteFileRust(const char *filePath, const void *buf, size_t bufSize, int isAppend);
    int ret = BlackboxFullWriteFileRust(filePath, buf, bufSize, isAppend);
    if (ret != 0) {
        BBOX_PRINT_ERR("write file [%s] failed!\n", filePath);
    }
    return ret;
#else
#ifdef LOSCFG_FS_VFS
    int fd;
    int totalToWrite = (int)bufSize;
    int totalWrite = 0;

    if (filePath == NULL || buf == NULL || bufSize == 0) {
        BBOX_PRINT_ERR("filePath: %p, buf: %p, bufSize: %lu!\n", filePath, buf, bufSize);
        return -1;
    }

    if (!IsLogPartReady()) {
        BBOX_PRINT_ERR("log path [%s] isn't ready to be written!\n", LOSCFG_BLACKBOX_LOG_ROOT_PATH);
        return -1;
    }
    fd = open(filePath, O_CREAT | O_RDWR | (isAppend ? O_APPEND : O_TRUNC), BBOX_FILE_MODE);
    if (fd < 0) {
        BBOX_PRINT_ERR("Create file [%s] failed, fd: %d!\n", filePath, fd);
        return -1;
    }
    while (totalToWrite > 0) {
        int writeThisTime = write(fd, buf, totalToWrite);
        if (writeThisTime < 0) {
            BBOX_PRINT_ERR("Failed to write file [%s]!\n", filePath);
            (void)close(fd);
            return -1;
        }
        buf += writeThisTime;
        totalToWrite -= writeThisTime;
        totalWrite += writeThisTime;
    }
    (void)fsync(fd);
    (void)close(fd);

    return (totalWrite == (int)bufSize) ? 0 : -1;
#else
    (VOID)filePath;
    (VOID)buf;
    (VOID)bufSize;
    (VOID)isAppend;
    return -1;
#endif
#endif
}

int SaveBasicErrorInfo(const char *filePath, const struct ErrorInfo *info)
{
#ifdef BLACKBOX_USE_RUST
    extern int BlackboxSaveBasicErrorInfoRust(const char *filePath, const struct ErrorInfo *info);
    int ret = BlackboxSaveBasicErrorInfoRust(filePath, info);
    if (ret != 0) {
        BBOX_PRINT_ERR("SaveBasicErrorInfo failed!\n");
    }
    return ret;
#else
    char *buf = NULL;

    if (filePath == NULL || info == NULL) {
        BBOX_PRINT_ERR("filePath: %p, event: %p!\n", filePath, info);
        return -1;
    }

    buf = LOS_MemAlloc(m_aucSysMem1, ERROR_INFO_MAX_LEN);
    if (buf == NULL) {
        BBOX_PRINT_ERR("LOS_MemAlloc failed!\n");
        return -1;
    }
    (void)memset_s(buf, ERROR_INFO_MAX_LEN, 0, ERROR_INFO_MAX_LEN);
    if (snprintf_s(buf, ERROR_INFO_MAX_LEN, ERROR_INFO_MAX_LEN - 1,
        ERROR_INFO_HEADER_FORMAT, info->event, info->module, info->errorDesc) != -1) {
        *(buf + ERROR_INFO_MAX_LEN - 1) = '\0';
        (void)FullWriteFile(filePath, buf, strlen(buf), 0);
    } else {
        BBOX_PRINT_ERR("buf is not enough or snprintf_s failed!\n");
    }

    (void)LOS_MemFree(m_aucSysMem1, buf);

    return 0;
#endif
}

#ifdef LOSCFG_FS_VFS
static int IsLogPartMounted(const char *devPoint, const char *mountPoint, struct statfs *statBuf, void *arg)
{
    (void)devPoint;
    (void)statBuf;
    (void)arg;
    if (mountPoint != NULL && arg != NULL) {
        if (strcmp(mountPoint, (char *)arg) == 0) {
            g_isLogPartReady = TRUE;
        }
    }
    return 0;
}

bool IsLogPartReady(void)
{
#ifdef BLACKBOX_USE_RUST
    extern int BlackboxIsLogPartReadyRust(int currentReady);
    if (!g_isLogPartReady) {
        (void)foreach_mountpoint((foreach_mountpoint_t)IsLogPartMounted, LOSCFG_BLACKBOX_LOG_PART_MOUNT_POINT);
    }
    return (BlackboxIsLogPartReadyRust((int)g_isLogPartReady) != 0);
#else
    if (!g_isLogPartReady) {
        (void)foreach_mountpoint((foreach_mountpoint_t)IsLogPartMounted, LOSCFG_BLACKBOX_LOG_PART_MOUNT_POINT);
    }
    return g_isLogPartReady;
#endif
}
#else
bool IsLogPartReady(void)
{
    return TRUE;
}
#endif

#ifdef LOSCFG_FS_VFS
int BlackboxAccess(const char *path)
{
    if (path == NULL) {
        return -1;
    }
    return access(path, 0);
}

int BlackboxMkdir(const char *path)
{
    if (path == NULL) {
        return -1;
    }
    return mkdir(path, BBOX_DIR_MODE);
}

int BlackboxOpenForWrite(const char *path, int isAppend)
{
    if (path == NULL) {
        return -1;
    }
    return open(path, O_CREAT | O_RDWR | (isAppend ? O_APPEND : O_TRUNC), BBOX_FILE_MODE);
}

int BlackboxWrite(int fd, const void *buf, size_t len)
{
    if (buf == NULL) {
        return -1;
    }
    return write(fd, buf, len);
}

int BlackboxFsync(int fd)
{
    return fsync(fd);
}

int BlackboxClose(int fd)
{
    return close(fd);
}

int CreateNewDir(const char *dirPath)
{
#ifdef BLACKBOX_USE_RUST
    extern int BlackboxCreateNewDirRust(const char *dirPath);
    int ret = BlackboxCreateNewDirRust(dirPath);
    if (ret != 0) {
        BBOX_PRINT_ERR("mkdir [%s] failed!\n", dirPath);
    }
    return ret;
#else
    int ret;

    if (dirPath == NULL) {
        BBOX_PRINT_ERR("dirPath is NULL!\n");
        return -1;
    }

    ret = access(dirPath, 0);
    if (ret == 0) {
        return 0;
    }
    ret = mkdir(dirPath, BBOX_DIR_MODE);
    if (ret != 0) {
        BBOX_PRINT_ERR("mkdir [%s] failed!\n", dirPath);
        return -1;
    }

    return 0;
#endif
}

int CreateLogDir(const char *dirPath)
{
#ifdef BLACKBOX_USE_RUST
    extern int BlackboxCreateLogDirRust(const char *dirPath);
    int ret = BlackboxCreateLogDirRust(dirPath);
    if (ret != 0) {
        BBOX_PRINT_ERR("Create log dir [%s] failed!\n", dirPath);
    }
    return ret;
#else
    const char *temp = dirPath;
    char curPath[PATH_MAX_LEN];
    int idx = 0;

    if (dirPath == NULL) {
        BBOX_PRINT_ERR("dirPath is NULL!\n");
        return -1;
    }
    if (*dirPath != '/') {
        BBOX_PRINT_ERR("Invalid dirPath: %s\n", dirPath);
        return -1;
    }
    (void)memset_s(curPath, sizeof(curPath), 0, sizeof(curPath));
    curPath[idx++] = *dirPath++;
    while (*dirPath != '\0' && idx < sizeof(curPath)) {
        if (*dirPath == '/') {
            if (CreateNewDir(curPath) != 0) {
                return -1;
            }
        }
        curPath[idx] = *dirPath;
        dirPath++;
        idx++;
    }
    if (*dirPath != '\0') {
        BBOX_PRINT_ERR("dirPath [%s] is too long!\n", temp);
        return -1;
    }

    return CreateNewDir(curPath);
#endif
}
#else
int BlackboxAccess(const char *path)
{
    (VOID)path;
    return -1;
}

int BlackboxMkdir(const char *path)
{
    (VOID)path;
    return -1;
}

int BlackboxOpenForWrite(const char *path, int isAppend)
{
    (VOID)path;
    (VOID)isAppend;
    return -1;
}

int BlackboxWrite(int fd, const void *buf, size_t len)
{
    (VOID)fd;
    (VOID)buf;
    (VOID)len;
    return -1;
}

int BlackboxFsync(int fd)
{
    (VOID)fd;
    return -1;
}

int BlackboxClose(int fd)
{
    (VOID)fd;
    return -1;
}

int CreateNewDir(const char *dirPath)
{
    (VOID)dirPath;
    return -1;
}

int CreateLogDir(const char *dirPath)
{
    (VOID)dirPath;
    return -1;
}
#endif
