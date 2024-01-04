# Host 流程

## 註冊 Hax / Woiden

Note1: 需先擁有 Telegram 帳號

Note2: 使用時盡量將 AdBlocker 關閉，支持無收費的 VPS 服務

1. 選擇 `Create VPS`，OS 可使用 `Debian`/`Ubuntu`

2. 完成人機驗證後至 `VPS Status` 等待開機

3. `Status` 顯示 `Online` 就能用 [`Web Terminal`](https://ssh.hax.co.id/) 連入

4. 輸入 `IP`/`密碼` 後連入

## 設置 WARP (可選)

如果不想使用 Web Terminal 管理，就需要設定 WARP (Woiden / Hax 使用 IPv6)

### 使用 Web Terminal 安裝 Warp

```bash
wget -N https://gitlab.com/fscarmen/warp/-/raw/main/menu.sh && bash menu.sh
```

### 連接 WireGuard

1. [WireGuard](https://www.wireguard.com/install/)

2. [線上提取設定檔](https://replit.com/@misaka-blog/wgcf-profile-generator)

3. 開啟 `WirdGuard`，左下角 `新增隧道`，將提取出來的文字貼上

4. 點擊 `連線` 按鈕

### 使用 vs code 連入

1. 安裝 [Remote SSH](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-ssh)

2. vs code command 選擇連入

   ```bash
   > ssh connect
   ```

   輸入 `root@[IPv6]`，並輸入密碼

## 下載 Bot Binary 並設定 .env

可以先建立一個資料夾

```bash
mkdir discord-bot && cd discord-bot
```

### 下載 Binary

從 [GitHub Release](https://github.com/JacKooDesu/serenity-discord-bot/releases/latest) 下載最新版本

```bash
wget https://github.com/JacKooDesu/serenity-discord-bot/releases/latest/download/serenity-discord-bot_linux.7z
```

需要使用 7z 解壓縮，如果有安裝可以省略這步

```bash
apt update && apt install p7zip-full
```

使用指令解壓縮

```bash
7z x serenity-discord-bot_linux.7z
```

要設定 binary 執行權限

```bash
chmod +x serenity-discord-bot
```

之後輸入 `ls -a -l` 應該能看到

```bash
total 39960
drwxr-xr-x 2 root root     4096 Jan  3 22:53 .
drwxr-xr-x 3 root root     4096 Jan  3 20:52 ..
-rw-r--r-- 1 root root      643 Jan  3 22:45 .env
-rwxr-xr-x 1 root root 33524992 Jan  3 12:27 serenity-discord-bot
-rw-r--r-- 1 root root  7377039 Jan  3 12:27 serenity-discord-bot_linux.7z
```

### 編輯 .env

- Note: 格式參考專案主頁 readme

1. 使用 vim 來編輯 `.env`

   ```bash
   vim .env
   ```

   複製 token 並貼上 (`vim` 的貼上快捷鍵為 `Ctrl + Shift + v`，輸入錯誤的話可以 `Esc` → `u` 復原)，完成後按兩次 `Esc`，輸入 `:wq` 保存離開

2. 使用 vscode 連入後編輯

   沒什麼好說的，跟平常編輯文件一樣

## 安裝 YT-DLP

1. 下載 yt-dlp

   ```bash
   wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/local/bin/yt-dlp
   ```

2. 設定執行權限

   ```bash
   chmod a+rx /usr/local/bin/yt-dlp
   ```

3. 測試指令

   ```bash
   yt-dlp --help
   ```

- Note: hax/woiden 環境本來就是 root，無須增加 `sudo`

## 使用 Screen 背景開啟

```bash
screen
```

```bash
exec ./senerity-discord-bot
```

- Note: `Ctrl + A` → `D` 離開當前 Screen
