# Mẫu Feature Request — CyberSkill v1

> Turn Your Will Into Real.

> Đây là bản dịch tiếng Việt của [README.md](./README.md). Bản tiếng Anh là bản chính thức; nếu hai bản khác nhau, bản tiếng Anh thắng.

## 1. Mục đích

Mẫu này ghi nhận đề xuất tính năng theo cách để product, engineering, và sales tranh luận trên cùng tập sự kiện, và đưa phân loại rủi ro EU AI Act lên sớm để ảnh hưởng đến thiết kế chứ không phải đến lúc phát hành.

## 2. Khi nào dùng mẫu này

Dùng khi đề xuất hành vi mới, mô-đun mới, integration mới, hoặc mở rộng phạm vi hiện có một cách đáng kể. Với defect, dùng mẫu Bug Report. Với một chỉnh sửa nhỏ một dòng không có hành vi nhìn thấy được với người dùng, một issue comment là đủ.

## 3. Tham chiếu các trường (frontmatter)

| Trường | Kiểu | Bắt buộc | Giá trị cho phép | Người điền | Lý do tồn tại |
|---|---|---|---|---|---|
| `title` | chuỗi | có | <= 72 ký tự | tác giả | Một dòng đề xuất |
| `author` | chuỗi | có | `@handle` | tác giả | Truy nguyên |
| `department` | enum | có | engineering, design, product, sales, operations, hr, client_success | tác giả | Định tuyến review |
| `status` | enum | có | draft .. closed | triage | Vòng đời |
| `priority` | enum | có | p0..p3 | triage | Mức ưu tiên công việc |
| `created_at` | chuỗi (ngày) | có | ISO 8601 | scaffolder | Mốc tính SLA review |
| `ai_authorship` | enum | có | none, assisted, co_authored, generated_then_reviewed | tác giả | Minh bạch |
| `template` | enum | có | `feature_request@1` | scaffolder | Khoá schema |
| `feature_type` | enum | có | user_facing, internal_tooling, integration, infrastructure | tác giả | Định tuyến và phân loại |
| `eu_ai_act_risk_class` | enum | có | not_ai, minimal, limited, high | tác giả | EU AI Act Điều 5–7. `unacceptable` không được phép |
| `target_release` | chuỗi | tuỳ chọn | SemVer hoặc quý (`2026-Q3`) | tác giả | Mốc roadmap |
| `client_visible` | boolean | có | true / false | tác giả | Kích hoạt yêu cầu Sales/CS Summary |

## 4. Tham chiếu các mục thân bài

| Mục | Bắt buộc? | Khi nào bắt buộc | Tốt trông như thế nào | Lỗi thường gặp |
|---|---|---|---|---|
| Summary | có | luôn luôn | Một đoạn lặp lại được từ trí nhớ | Một danh sách mong muốn |
| Problem | có | luôn luôn | Bằng chứng có trích dẫn: ticket, NPS, telemetry | Người dùng giả định |
| Customer Quotes | có điều kiện | `client_visible=true` | Nguyên văn, có ghi nguồn nếu có thể | Diễn giải lại |
| Proposed Solution | có | luôn luôn | Hành vi nhìn thấy được, không phải chi tiết hiện thực | Chi tiết hiện thực |
| Alternatives Considered | có | luôn luôn | Bạn loại bỏ gì và tại sao | "Chúng tôi không cân nhắc gì khác" |
| Success Metrics | có | luôn luôn | Một metric chính + một guardrail | Đếm số phù phiếm |
| Scope | có | luôn luôn | Danh sách out-of-scope rõ ràng | Ranh giới mơ hồ |
| Dependencies | có | luôn luôn | Mô-đun, đội ngũ, vendor khác | "Không có" |
| AI Risk Assessment | có điều kiện | `eu_ai_act_risk_class` là `limited` hoặc `high` | Ba subsection điền đầy đủ | Bỏ Failure Modes |
| Sales/CS Summary | có điều kiện | `client_visible=true` | Một đoạn để non-engineer pitch được | Thuật ngữ nội bộ, mã mô-đun |
| AI Authorship Disclosure | có điều kiện | `ai_authorship != none` | Đủ ba bullet | Phạm vi mơ hồ |

## 5. Quy tắc bắt buộc theo điều kiện

Validator áp dụng:

1. `eu_ai_act_risk_class` là `limited` hoặc `high` ⇒ mục `## AI Risk Assessment` với các subsection `### Data Sources`, `### Human Oversight`, `### Failure Modes`.
2. `client_visible: true` ⇒ mục `## Customer Quotes` và `## Sales/CS Summary` có mặt.
3. `ai_authorship != none` ⇒ mục `## AI Authorship Disclosure` đủ ba bullet.
4. Schema từ chối thẳng `eu_ai_act_risk_class: unacceptable` — tính năng thuộc nhóm này không được phép file.

## 6. Ví dụ (feature đã điền đầy đủ)

Xem ví dụ Markdown trong bản tiếng Anh: [README.md mục 6](./README.md#6-example-fully-filled-realistic-artifact). Thân feature request thực tế là tiếng Anh; phần tiếng Việt chỉ phục vụ tài liệu.

## 7. Phản mẫu (anti-patterns)

- Đặt `eu_ai_act_risk_class: minimal` để né AI Risk Assessment khi tính năng thực sự phát ra nội dung AI cho khách hàng. Phân loại đúng là `limited` (nghĩa vụ minh bạch theo Điều 50).
- Bỏ qua `Alternatives Considered` vì "câu trả lời rõ ràng". Nếu rõ ràng thì viết một câu — vẫn phải viết.
- `client_visible: true` mà không có block Customer Quotes. Validator bắt; đừng lách bằng cách nói dối cờ.

## 8. Sử dụng theo phòng ban

| Phòng ban | Điền gì | Bỏ qua gì |
|---|---|---|
| Product | Mọi mục; bạn sở hữu spec | (không có) |
| Engineering | Thường co-author với Product; hữu ích ở Dependencies và Failure Modes | Sales/CS Summary trừ khi viết hộ Product |
| Sales / CS | Customer Quotes (nguyên văn), bằng chứng Problem, Sales/CS Summary | Chi tiết hiện thực ở Proposed Solution |
| Design | Có thể author tính năng user-facing; mockup ở Proposed Solution | Compliance trừ khi có ngữ cảnh pháp lý |

## 9. Bản tiếng Anh

Tài liệu gốc tiếng Anh nằm tại [README.md](./README.md). Hai tệp được giữ đồng bộ thủ công; nếu sửa một bản, sửa luôn bản còn lại. Khi có khác biệt, bản tiếng Anh thắng.

Thân feature request là tiếng Anh. Nếu tính năng được ship cho người dùng tiếng Việt, ship phiên bản bản địa hoá của các chuỗi user-facing tại bề mặt nó xuất hiện (chuỗi trong sản phẩm, copy marketing). Bản thân feature request giữ nguyên tiếng Anh.

## 10. Ghi chú tuân thủ

EU AI Act Điều 5–7 quy định phân loại rủi ro; Điều 14 quy định human oversight; Điều 50 quy định nghĩa vụ minh bạch cho nội dung AI hiển thị cho người tự nhiên. Trường `eu_ai_act_risk_class` cộng với mục `## AI Risk Assessment` (bắt buộc theo điều kiện) là sự thực thi cấu trúc.

Xem [docs/compliance/eu-ai-act-risk-classes.md](../../docs/compliance/eu-ai-act-risk-classes.md) để biết ánh xạ đầy đủ và quy tắc chọn nhóm.

## 11. Hướng dẫn về AI authorship cho artifact này

Trên thực tế feature request thường được hỗ trợ AI nặng — điều này ổn và khuyến khích. Đặt `ai_authorship` chính xác và điền block disclosure. Disclosure là theo artifact, không theo công cụ: nếu Claude viết văn bản và Cursor refactor một bản phác mã inline, liệt kê cả hai.

## 12. Di trú từ phiên bản v1.0 cũ

Xem [docs/migration/from-v1-yaml-forms.md](../../docs/migration/from-v1-yaml-forms.md).

## 13. Hợp đồng validation (validator kiểm tra gì)

Validator áp dụng:

- Tất cả trường frontmatter bắt buộc đều có mặt.
- Khoá frontmatter là snake_case.
- Giá trị enum nằm trong tập cho phép.
- `eu_ai_act_risk_class` không thể là `unacceptable` (schema từ chối).
- `eu_ai_act_risk_class` là `limited` hoặc `high` ⇒ mục `## AI Risk Assessment` với ba subsection bắt buộc.
- `client_visible: true` ⇒ mục `## Customer Quotes` và `## Sales/CS Summary` có mặt.
- `ai_authorship != none` ⇒ mục `## AI Authorship Disclosure` đủ ba bullet.
- Block `<untrusted_content>` không lồng nhau và không chứa dấu hiệu prompt-injection.

Mã thoát: `0` đạt, `1` có lỗi, `2` chỉ có cảnh báo.
