# MoliAtri

为[AtriBot](https://github.com/LaoLittle/atri_bot)接入茉莉云机器人

### 配置文件

配置文件位于`atri_bot/workspaces/moli_atri/config.toml`

其中api_key与api_secret均需要从[茉莉云官网](https://www.mlyai.com)获取
```toml
api_key = ''
api_secret = ''
name = '亚托莉'
reply_times = 1
do_quote_reply = false # 是否引用回复(暂时无效, 等待后续更新)
do_print_results_on_console = true
default_reply = [
    '？',
    '怎么',
    '怎么了',
    '什么？',
    '在',
    '嗯？',
]
timeout_reply = [
    '没事我就溜了',
    'emmmmm',
    '......',
    '溜了',
    '？',
]
```

本插件需要AtriBot >= v0.3.1