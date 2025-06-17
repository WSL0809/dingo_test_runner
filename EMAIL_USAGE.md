# 邮件通知功能使用指南

## 功能概述

MySQL Test Runner 现在支持通过邮件发送测试报告，包括：

- 🎨 **美观的HTML报告** - 响应式设计，支持深浅色模式
- 📊 **详细的统计信息** - 通过率、失败数、执行时间等
- 📎 **JUnit XML附件** - 可选附带XML报告文件
- 📧 **多收件人支持** - 支持发送给多个邮箱地址
- 🔒 **TLS安全连接** - 支持加密的SMTP连接

## 快速开始

### 1. 编译带邮件功能的版本

```bash
cargo build --features email --release
```

### 2. 基本使用示例

```bash
# 运行测试并发送邮件报告
./target/release/dingo_test_runner \
  --email-enable \
  --email-smtp-host smtp.gmail.com \
  --email-smtp-port 587 \
  --email-username your-email@gmail.com \
  --email-password your-app-password \
  --email-to recipient1@example.com,recipient2@example.com \
  --email-enable-tls \
  --xunit-file test_report.xml \
  simple_test
```

## 邮件配置参数

| 参数 | 说明 | 默认值 | 必需 |
|------|------|--------|------|
| `--email-enable` | 启用邮件通知 | false | ✅ |
| `--email-smtp-host` | SMTP服务器地址 | - | ✅ |
| `--email-smtp-port` | SMTP端口 | 587 | - |
| `--email-username` | 邮箱用户名 | - | ✅ |
| `--email-password` | 邮箱密码/应用密码 | - | ✅ |
| `--email-to` | 收件人邮箱(逗号分隔) | - | ✅ |
| `--email-from` | 发件人显示名称 | "MySQL Tester" | - |
| `--email-enable-tls` | 启用TLS加密 | false | 推荐 |

## 常用邮箱配置

### Gmail

```bash
--email-smtp-host smtp.gmail.com \
--email-smtp-port 587 \
--email-username your-email@gmail.com \
--email-password your-app-password \
--email-enable-tls
```

> **注意**: Gmail需要使用应用密码，不能使用账户密码。请在Google账户设置中生成应用密码。

### Outlook/Hotmail

```bash
--email-smtp-host smtp-mail.outlook.com \
--email-smtp-port 587 \
--email-username your-email@outlook.com \
--email-password your-password \
--email-enable-tls
```

### QQ邮箱

```bash
--email-smtp-host smtp.qq.com \
--email-smtp-port 587 \
--email-username your-email@qq.com \
--email-password your-authorization-code \
--email-enable-tls
```

### 企业邮箱示例

```bash
--email-smtp-host mail.company.com \
--email-smtp-port 587 \
--email-username user@company.com \
--email-password your-password \
--email-enable-tls
```

## 完整使用示例

### 运行单个测试并发送报告

```bash
cargo run --features email -- \
  --email-enable \
  --email-smtp-host smtp.gmail.com \
  --email-smtp-port 587 \
  --email-username test-runner@gmail.com \
  --email-password abcd-efgh-ijkl-mnop \
  --email-to dev-team@company.com,qa-team@company.com \
  --email-from "MySQL测试机器人" \
  --email-enable-tls \
  --xunit-file daily_report.xml \
  simple_test
```

### 运行所有测试并发送报告

```bash
cargo run --features email -- \
  --all \
  --email-enable \
  --email-smtp-host smtp.company.com \
  --email-username ci@company.com \
  --email-password ci-password \
  --email-to team@company.com \
  --email-enable-tls \
  --xunit-file full_test_report.xml
```

## 邮件报告内容

### HTML报告包含：

1. **统计概览卡片**
   - 通过测试数量
   - 失败测试数量
   - 总测试数量
   - 执行总时间

2. **详细测试列表**
   - 测试名称
   - 执行状态（通过/失败）
   - 执行时间
   - 错误信息（如有）

3. **美观的视觉设计**
   - 响应式布局
   - 状态颜色编码
   - 专业的样式

### 纯文本备用版本

如果收件人的邮件客户端不支持HTML，会自动提供纯文本版本的报告。

## 故障排除

### 常见问题

1. **认证失败**
   - 确认用户名和密码正确
   - Gmail用户需要使用应用密码
   - 检查是否启用了两步验证

2. **连接超时**
   - 检查SMTP服务器地址和端口
   - 确认网络连接正常
   - 尝试不同的端口（25, 465, 587）

3. **TLS错误**
   - 确认邮箱服务商支持TLS
   - 尝试关闭TLS（不推荐）
   - 检查防火墙设置

### 调试模式

使用详细日志查看邮件发送过程：

```bash
cargo run --features email -- \
  --log-level info \
  --email-enable \
  [其他参数...] \
  simple_test
```

### 测试邮件配置

建议先用简单的测试验证邮件配置：

```bash
cargo run --features email -- \
  --email-enable \
  --email-smtp-host your-smtp-host \
  --email-username your-username \
  --email-password your-password \
  --email-to your-test-email@example.com \
  --email-enable-tls \
  simple_test
```

## 安全建议

1. **使用应用密码**: 避免使用主账户密码
2. **环境变量**: 可以通过环境变量传递敏感信息
3. **TLS加密**: 始终启用TLS保护传输安全
4. **权限控制**: 限制邮箱账户的权限范围

## 环境变量支持

为了安全起见，可以通过环境变量设置敏感信息：

```bash
export EMAIL_USERNAME="your-email@gmail.com"
export EMAIL_PASSWORD="your-app-password"

cargo run --features email -- \
  --email-enable \
  --email-smtp-host smtp.gmail.com \
  --email-username "$EMAIL_USERNAME" \
  --email-password "$EMAIL_PASSWORD" \
  --email-to team@company.com \
  --email-enable-tls \
  simple_test
```

## CI/CD集成

在CI/CD流水线中使用：

```yaml
# GitHub Actions 示例
- name: Run tests with email notification
  run: |
    cargo run --features email -- \
      --all \
      --email-enable \
      --email-smtp-host ${{ secrets.SMTP_HOST }} \
      --email-username ${{ secrets.EMAIL_USERNAME }} \
      --email-password ${{ secrets.EMAIL_PASSWORD }} \
      --email-to ${{ secrets.EMAIL_RECIPIENTS }} \
      --email-enable-tls \
      --xunit-file ci_test_report.xml
```

---

如有问题或建议，请联系开发团队！ 