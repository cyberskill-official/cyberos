# Mẫu Pull Request — CyberSkill v1

> Turn Your Will Into Real.

> Đây là bản dịch tiếng Việt của [README.md](./README.md). Bản tiếng Anh là bản chính thức; nếu hai bản khác nhau, bản tiếng Anh thắng.

## 1. Mục đích

Mẫu này là hợp đồng mà mọi pull request mở trong kho mã CyberSkill phải tuân thủ. Mục tiêu: rút ngắn thời gian review, tạo bằng chứng audit-able cho SOC 2 CC8.1, và để lại dấu vết rõ ràng về việc sử dụng AI trong tác phẩm mà không làm chậm con người. Đây là mẫu PR chính thức; các phòng ban không tự fork riêng.

## 2. Khi nào dùng mẫu này

Dùng cho mọi PR có thay đổi mã nguồn — `feat`, `fix`, `refactor`, `perf`, `chore`, v.v. Cũng dùng cho PR tài liệu (`docs`) và CI (`ci`) với phần lớn trường để mặc định. Ngoại lệ duy nhất là PR tự động do dependabot/renovate tạo, vốn có mô tả định dạng máy riêng.

## 3. Tham chiếu các trường (frontmatter)

| Trường | Kiểu | Bắt buộc | Giá trị cho phép | Người điền | Lý do tồn tại |
|---|---|---|---|---|---|
| `title` | chuỗi | có | <= 72 ký tự, theo Conventional Commits | tác giả | Bắt buộc một câu mô tả ý định; dùng cho changelog |
| `author` | chuỗi | có | `@handle` (GitHub) | tác giả | Truy nguyên, định tuyến CODEOWNERS |
| `department` | enum | có | engineering, design, product, sales, operations, hr, client_success | tác giả | Định tuyến tự động hoá review; báo cáo liên phòng ban |
| `status` | enum | có | draft, ready_for_review, in_review, approved, merged, closed | tác giả / reviewer | Tín hiệu vòng đời, độc lập với trạng thái GitHub |
| `priority` | enum | có | p0, p1, p2, p3 | tác giả | Mức ưu tiên công việc; khác với severity |
| `created_at` | chuỗi (ngày) | có | ISO 8601 `YYYY-MM-DD` | scaffolder | Mốc tính SLA |
| `ai_authorship` | enum | có | none, assisted, co_authored, generated_then_reviewed | tác giả | Minh bạch theo EU AI Act Điều 50 |
| `template` | enum | có | `pull_request@1` | scaffolder | Khoá schema để PR cũ không hỏng khi schema mới ra |
| `pr_type` | enum | có | feat, fix, docs, refactor, perf, test, build, ci, chore, revert | tác giả | Định tuyến theo Conventional Commits |
| `breaking_change` | boolean | có | true / false | tác giả | Kích hoạt yêu cầu mục `## Migration` |
| `linked_issues` | mảng | tuỳ chọn | `#123` hoặc `org/repo#123` | tác giả | Liên kết chéo |
| `soc2_change_class` | enum | có | standard, expedited, emergency | tác giả | Phân loại theo SOC 2 CC8.1 |

## 4. Tham chiếu các mục thân bài

| Mục | Bắt buộc? | Khi nào bắt buộc | Tốt trông như thế nào | Lỗi thường gặp |
|---|---|---|---|---|
| Summary | có | luôn luôn | Hai câu giải thích ý định, không tham chiếu đến tệp | "Refactor X" không có lý do |
| Context | có | luôn luôn | Liên kết tới issue/doc, kèm một đoạn văn bản | Chỉ có một đường dẫn ticket |
| Changes | có | luôn luôn | Danh sách nhóm theo khu vực, không liệt kê từng tệp | Dán `git diff --stat` |
| How to verify | có | luôn luôn | Lệnh cụ thể và đầu ra mong đợi | "Đã test cục bộ" |
| Risk and rollback | có | luôn luôn | Nêu tên migration rollback hoặc feature flag | "Rủi ro thấp" mà không có kế hoạch |
| Migration | có điều kiện | `breaking_change=true` | Mã trước/sau, mục tiêu phiên bản | Để trống khi cờ đã set |
| Post-Incident Review Plan | có điều kiện | `soc2_change_class=emergency` | Owner, ngày, ticket, tham chiếu control | Bỏ tham chiếu SOC 2 |
| AI Authorship Disclosure | có điều kiện | `ai_authorship != none` | Đủ ba bullet, không thừa | Khẳng định "AI gõ hộ" mà không nêu phạm vi |

## 5. Quy tắc bắt buộc theo điều kiện

Validator áp dụng các quy tắc sau — vi phạm là chặn merge, không phải cảnh báo:

1. `breaking_change: true` ⇒ thân bài có mục `## Migration` với ít nhất một đoạn không trống.
2. `soc2_change_class: emergency` ⇒ thân bài có mục `## Post-Incident Review Plan`.
3. `ai_authorship != none` ⇒ thân bài có mục `## AI Authorship Disclosure` đủ ba bullet.

## 6. Ví dụ (PR thực tế đã điền đầy đủ)

Xem ví dụ Markdown trong bản tiếng Anh: [README.md mục 6](./README.md#6-example-fully-filled-realistic-artifact). Thân bài PR thực tế là tiếng Anh — phần tiếng Việt chỉ phục vụ mục đích tài liệu này.

## 7. Phản mẫu (anti-patterns)

- "Thay đổi nhỏ, không cần test" mà không nói rõ tại sao nhỏ. Đáng nói thì nói; không đáng nói thì viết test.
- Mục Migration với `breaking_change: true` mà không có ví dụ mã — validator chặn; đừng tìm cách lách.
- Khai `ai_authorship: none` khi AI viết phần mô tả. Disclosure tồn tại cho cả văn bản, không chỉ cho mã.
- `soc2_change_class: standard` cho hot-fix bỏ qua review. Nếu đã bỏ qua review thì phân loại `expedited` hoặc `emergency` và chịu trách nhiệm review hậu kỳ.

## 8. Sử dụng theo phòng ban

| Phòng ban | Điền gì | Bỏ qua gì |
|---|---|---|
| Engineering | Mọi trường kỹ thuật, toàn bộ thân bài | (không có) |
| Design | `department: design`, thân bài tập trung vào thay đổi UX; liên kết mockup ở Context | Verification chuyên về mã |
| Product | `department: product`, dùng cho PR spec đối với kho product | Migration trừ khi spec phá vỡ công cụ phụ thuộc |
| Sales / CS | Hầu như không mở PR — mở issue thay vào đó | Phần lớn trường |

## 9. Bản tiếng Anh

Tài liệu gốc tiếng Anh nằm tại [README.md](./README.md). Hai tệp được giữ đồng bộ thủ công; nếu sửa một bản, sửa luôn bản còn lại. Khi có khác biệt giữa hai bản, bản tiếng Anh là bản chính thức.

Bản thân thân PR là tiếng Anh. Bản địa hoá tiếng Việt thuộc về tài liệu, không thuộc về artifact mà validator phân tích.

## 10. Ghi chú tuân thủ

Mẫu này mang theo bằng chứng change-management cho SOC 2 CC8.1. Trường `soc2_change_class` cộng với mục `## Post-Incident Review Plan` (bắt buộc theo điều kiện) là dấu vết audit. Đừng xoá các trường này khỏi mẫu kể cả với PR mà bạn nghĩ là ngoài phạm vi.

Xem [docs/compliance/soc2-change-management.md](../../docs/compliance/soc2-change-management.md) để biết ánh xạ đầy đủ.

## 11. Hướng dẫn về AI authorship cho artifact này

Nếu AI tham gia viết phần mô tả PR, mã, test, hoặc ghi chú migration, đặt `ai_authorship` ở mức phù hợp nhất trong `assisted`, `co_authored`, hoặc `generated_then_reviewed`, và điền đủ block disclosure ba bullet. Disclosure không phải lời thú nhận; nó là tuyên bố về phạm vi.

## 12. Di trú từ phiên bản v1.0 cũ

Xem [docs/migration/from-v1-pr-template.md](../../docs/migration/from-v1-pr-template.md).

## 13. Hợp đồng validation (validator kiểm tra gì)

Validator (`@cyberskill/templates validate`) áp dụng:

- Tất cả trường frontmatter bắt buộc đều có mặt.
- Khoá frontmatter là snake_case (không kebab-case, không camelCase).
- Giá trị enum nằm trong tập cho phép.
- `breaking_change=true` ⇒ mục `## Migration` có ít nhất một đoạn không trống.
- `soc2_change_class=emergency` ⇒ mục `## Post-Incident Review Plan` có mặt.
- `ai_authorship != none` ⇒ mục `## AI Authorship Disclosure` đủ ba bullet.
- `title` phân tích được như subject Conventional Commits khi có cờ `--pr-title`.

Mã thoát: `0` đạt, `1` có lỗi (chặn merge), `2` chỉ có cảnh báo.
