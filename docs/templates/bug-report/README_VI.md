# Mẫu Bug Report — CyberSkill v1

> Turn Your Will Into Real.

> Đây là bản dịch tiếng Việt của [README.md](./README.md). Bản tiếng Anh là bản chính thức; nếu hai bản khác nhau, bản tiếng Anh thắng.

## 1. Mục đích

Mẫu này ghi nhận lỗi theo cách lặp lại được cho engineering, có thể bảo vệ trước cơ quan tuân thủ (PDPL Việt Nam Điều 23), và an toàn với công cụ AI triage. Đồng hồ thông báo 72 giờ về vi phạm dữ liệu được mã hoá vào cấu trúc để không thể bỏ sót do quên.

## 2. Khi nào dùng mẫu này

Dùng cho mọi defect — lỗi hành vi, lỗi tài liệu, lỗi accessibility, lỗi bảo mật, suy giảm hiệu năng. Nếu hệ thống đang làm việc đáng lẽ không nên làm, hoặc không làm việc đáng lẽ phải làm, đây là mẫu cần dùng.

Với một tính năng mong muốn, dùng mẫu Feature Request.

## 3. Tham chiếu các trường (frontmatter)

| Trường | Kiểu | Bắt buộc | Giá trị cho phép | Người điền | Lý do tồn tại |
|---|---|---|---|---|---|
| `title` | chuỗi | có | <= 72 ký tự | người báo | Một dòng mô tả defect |
| `author` | chuỗi | có | `@handle` | người báo | Truy nguyên |
| `department` | enum | có | engineering, design, product, sales, operations, hr, client_success | người báo | Định tuyến triage |
| `status` | enum | có | draft .. closed | triage | Vòng đời |
| `priority` | enum | có | p0..p3 | triage | Mức ưu tiên công việc |
| `created_at` | chuỗi (ngày) | có | ISO 8601 | scaffolder | Mốc tính SLA |
| `ai_authorship` | enum | có | none, assisted, co_authored, generated_then_reviewed | người báo | Minh bạch |
| `template` | enum | có | `bug_report@1` | scaffolder | Khoá schema |
| `severity` | enum | có | sev1, sev2, sev3, sev4 | triage | Mức tác động đến khách hàng, khác với priority |
| `affected_versions` | mảng | có | Dải SemVer, ít nhất một | người báo | Khoanh vùng tìm kiếm |
| `pdpl_breach_suspected` | boolean | có | true / false | người báo | Kích hoạt các trường có điều kiện theo PDPL Điều 23 |
| `discovered_at` | datetime | có điều kiện | ISO 8601 có timezone | người báo | Bắt buộc khi `pdpl_breach_suspected=true`; mốc bắt đầu đồng hồ |
| `reproducible` | enum | có | always, intermittent, once, unable_to_reproduce | người báo | Tín hiệu triage |

## 4. Tham chiếu các mục thân bài

| Mục | Bắt buộc? | Khi nào bắt buộc | Tốt trông như thế nào | Lỗi thường gặp |
|---|---|---|---|---|
| Summary | có | luôn luôn | Triệu chứng + khu vực, hai câu | Dán stack trace |
| Reporter Description | có | luôn luôn | Nguyên văn; trong block `<untrusted_content>` nếu từ khách hàng | Diễn giải lại lời khách |
| Steps to Reproduce | có | luôn luôn | Đánh số, copy-paste chạy được | "Cứ mở trang là thấy" |
| Expected Behaviour | có | luôn luôn | Tham chiếu spec hoặc hành vi cũ | Quan điểm cá nhân |
| Actual Behaviour | có | luôn luôn | Thông báo lỗi nguyên văn + ảnh chụp | "Bị crash" |
| Environment | có | luôn luôn | Phiên bản, OS, region, tenant ID | "Trên laptop tôi" |
| Impact | có | luôn luôn | Số lượng, doanh thu, hợp đồng | "Quan trọng" |
| Breach Containment | có điều kiện | `pdpl_breach_suspected=true` | Hành động ngăn chặn ngay, mức phơi nhiễm còn lại | Để điền sau |
| Notification Plan | có điều kiện | `pdpl_breach_suspected=true` | Đối tượng dữ liệu, cơ quan, hạn (tính từ `discovered_at`) | Mốc thời gian mơ hồ |
| AI Authorship Disclosure | có điều kiện | `ai_authorship != none` | Đủ ba bullet | Bỏ bullet "Human review" |

## 5. Quy tắc bắt buộc theo điều kiện

Validator áp dụng:

1. `pdpl_breach_suspected: true` ⇒ `discovered_at` được đặt (do schema kiểm tra) VÀ thân bài có mục `## Breach Containment` VÀ `## Notification Plan` (do validator kiểm tra).
2. `ai_authorship != none` ⇒ mục `## AI Authorship Disclosure` đủ ba bullet.

## 6. Ví dụ (bug đã điền đầy đủ)

Xem ví dụ Markdown trong bản tiếng Anh: [README.md mục 6](./README.md#6-example-fully-filled-realistic-artifact). Thân bug report thực tế là tiếng Anh; phần tiếng Việt chỉ phục vụ tài liệu.

## 7. Phản mẫu (anti-patterns)

- Đặt `pdpl_breach_suspected: false` để né các mục thêm. Validator không bắt được lời nói dối, nhưng auditor sẽ bắt. Khi nghi ngờ, đặt true và để Legal đánh giá.
- Dán nội dung email khách hàng ngoài block `<untrusted_content>`. Block này là biên giới mà các biện pháp chống prompt injection dựa vào.
- Đặt `severity: sev1` mà không đặt `priority: p0`. Severity là mức tác động; priority là thứ tự công việc. Thường khớp nhau, nếu không thì giải thích trong mục Impact.

## 8. Sử dụng theo phòng ban

| Phòng ban | Điền gì | Bỏ qua gì |
|---|---|---|
| Engineering | Mọi trường kỹ thuật, toàn bộ thân bài | (không có) |
| Client Success | `## Reporter Description` (nguyên văn từ khách), cờ `pdpl_breach_suspected`, Impact | Repro kỹ thuật chi tiết — để Eng làm |
| Sales | Severity từ góc nhìn khách hàng, tác động kinh doanh | Stack trace, `affected_versions` |
| Design | Lỗi UI/accessibility: ảnh chụp, mong đợi vs thực tế, tên component | Repro phía backend |
| Operations | Sự cố hạ tầng: tenant ID, region, link observability | Repro mức mã |

Đây là thực tế của "consultancy mười người" — Sales và CS có lúc phải file bug.

## 9. Bản tiếng Anh

Tài liệu gốc tiếng Anh nằm tại [README.md](./README.md). Hai tệp được giữ đồng bộ thủ công; nếu sửa một bản, sửa luôn bản còn lại. Khi có khác biệt, bản tiếng Anh thắng.

Thân bug report là tiếng Anh. Văn bản tiếng Việt cho post-mortem hướng đến khách hàng nên viết ở tài liệu riêng — đừng nhồi vào template này.

## 10. Ghi chú tuân thủ

PDPL Việt Nam Điều 23: thông báo trong vòng 72 giờ kể từ khi phát hiện vi phạm dữ liệu cá nhân. Cờ `pdpl_breach_suspected`, trường `discovered_at` (bắt buộc theo điều kiện), và các mục `## Breach Containment` + `## Notification Plan` (bắt buộc theo điều kiện) là sự thực thi cấu trúc của nghĩa vụ này.

Xem [docs/compliance/pdpl-vietnam-breach-clock.md](../../docs/compliance/pdpl-vietnam-breach-clock.md) để biết toàn văn và quy tắc tính hạn.

## 11. Hướng dẫn về AI authorship cho artifact này

Bug report soạn với hỗ trợ AI phải khai báo. AI đặc biệt hữu ích trong việc viết lại tin nhắn ngắn của khách hàng thành các bước repro — khi làm vậy, đặt `ai_authorship: assisted` và nêu tên công cụ. Mục Reporter Description vẫn phải giữ nguyên văn.

## 12. Di trú từ phiên bản v1.0 cũ

Xem [docs/migration/from-v1-yaml-forms.md](../../docs/migration/from-v1-yaml-forms.md).

## 13. Hợp đồng validation (validator kiểm tra gì)

Validator áp dụng:

- Tất cả trường frontmatter bắt buộc đều có mặt.
- Khoá frontmatter là snake_case.
- Giá trị enum nằm trong tập cho phép.
- `pdpl_breach_suspected=true` ⇒ `discovered_at` được đặt (schema), VÀ các mục `## Breach Containment` và `## Notification Plan` có mặt (body).
- `ai_authorship != none` ⇒ mục `## AI Authorship Disclosure` đủ ba bullet.
- Block `<untrusted_content>` không lồng nhau và không chứa dấu hiệu prompt-injection.

Mã thoát: `0` đạt, `1` có lỗi, `2` chỉ có cảnh báo.
