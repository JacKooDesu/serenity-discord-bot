# Host 流程

## 註冊 Hax / Woiden

Note: 需先擁有 Telegram 帳號

## 使用 Web Terminal 安裝 Warp

```bash
wget -N https://gitlab.com/fscarmen/warp/-/raw/main/menu.sh && bash menu.sh
```

## 連接 WireGuard

1. [WireGuard](https://www.wireguard.com/install/)

2. [線上提取設定檔](https://replit.com/@misaka-blog/wgcf-profile-generator)

3. 開啟 `WirdGuard`，左下角 `新增隧道`，將提取出來的文字貼上

4. 點擊 `連線` 按鈕

## 使用 vs code 連入

1. 安裝 [Remote SSH](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-ssh)

2. 連入

    ```bash
    > ssh connect
    ```

    輸入 ipv6 位址

## 下載/上傳 Bot Binary

## 安裝 YT-DLP

1. 下載 yt-dlp

   ```bash
   sudo wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/local/bin/yt-dlp
   ```

2. 設定執行權限

   ```bash
   sudo chmod a+rx /usr/local/bin/yt-dlp
   ```

3. 測試指令

   ```bash
   yt-dlp --help
   ```

- Note: hax/woiden 環境本來就是 root，無須再指令前增加 `sudo`

## 使用 Screen 背景開啟

```bash
screen
```

```bash
exec ./senerity-discord-bot
```

- Note: `Ctrl + A` -> `D` 離開當前 Screen
