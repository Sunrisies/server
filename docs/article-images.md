# 文章图片上传

前端通过 `POST /api/v1/images/upload` 发送multipart/form-data，字段名 `file`。仍由现有鉴权中间件要求管理员身份。

默认使用已有七牛云方式（`IMAGE_STORAGE_PROVIDER=qiniu` 或不设置）。需要配置实际七牛凭据、bucket和domain。未配置时返回503及提示；本次未使用真实七牛账户进行云上传。

本地部署可显式启用：

```dotenv
IMAGE_STORAGE_PROVIDER=local
LOCAL_IMAGE_DIR=/absolute/persistent/path/article-images
```

local图片上传限制10MiB，验证PNG/JPEG/WebP真实内容和解码资源，在阻塞线程中处理，超过1920×1080的图片按比例缩小，统一存为WebP。返回的相对地址为 `/api/v1/images/file/{uuid}.webp`。通过Nuxt同源代理访问，因此不要将该接口当作public目录中的文件交给IPX读取；前端articleImages provider已做对应处理，其他静态图片仍走原IPX。

文件名由服务端UUID生成，读取接口只接受规范UUID.webp。响应为image/webp并设置nosniff。图片写入已有images表，保留原有上传管理记录。images记录保存失败时清理刚写入的本地文件。上传管理删除识别local键，不会误调用七牛。

本地图片URL是公开资源，草稿文章正文不公开不等于媒体URL具有私有访问控制。本方案不实现私有附件或按文章授权的图片访问。已有图片素材库列表/详情/删除存根接口不属于本次编辑器接入范围。

部署时必须持久化LOCAL_IMAGE_DIR并与数据库一同备份，切换服务目录时不要丢弃图片目录。多实例应共享媒体目录或使用对象存储。本次只在回环地址本地验证，未修改线上配置。
