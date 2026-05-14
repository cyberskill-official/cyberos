# Vietnamese bank BIN codes (Napas)

The 6-digit Bank Identification Number (BIN) is assigned by Napas — Vietnam's national interbank switch. Every Napas247 transfer routes by BIN; the bank app at the receiving end maps the BIN back to the institution.

The 20 banks below cover the overwhelming majority of retail and SME transaction volume in Vietnam (estimated >97%).

| Short code | BIN     | Full name (Vietnamese)                                                         | Full name (English)                                | HQ city     |
|------------|---------|--------------------------------------------------------------------------------|---------------------------------------------------|-------------|
| VCB        | 970436  | Ngân hàng TMCP Ngoại thương Việt Nam                                          | Joint Stock Commercial Bank for Foreign Trade of Vietnam | Hà Nội  |
| BIDV       | 970418  | Ngân hàng TMCP Đầu tư và Phát triển Việt Nam                                  | Bank for Investment and Development of Vietnam     | Hà Nội      |
| CTG        | 970415  | Ngân hàng TMCP Công thương Việt Nam (VietinBank)                              | Vietnam Joint Stock Commercial Bank for Industry and Trade | Hà Nội |
| AGRIBANK   | 970405  | Ngân hàng Nông nghiệp và Phát triển Nông thôn Việt Nam                        | Vietnam Bank for Agriculture and Rural Development | Hà Nội      |
| TCB        | 970407  | Ngân hàng TMCP Kỹ thương Việt Nam (Techcombank)                               | Vietnam Technological and Commercial Joint Stock Bank | Hà Nội  |
| MB         | 970422  | Ngân hàng TMCP Quân đội                                                       | Military Commercial Joint Stock Bank               | Hà Nội      |
| ACB        | 970416  | Ngân hàng TMCP Á Châu                                                         | Asia Commercial Joint Stock Bank                   | TP.HCM      |
| STB        | 970403  | Ngân hàng TMCP Sài Gòn Thương Tín (Sacombank)                                 | Saigon Thuong Tin Commercial Joint Stock Bank      | TP.HCM      |
| VPB        | 970432  | Ngân hàng TMCP Việt Nam Thịnh Vượng (VPBank)                                  | Vietnam Prosperity Joint Stock Commercial Bank     | Hà Nội      |
| HDB        | 970437  | Ngân hàng TMCP Phát triển TP.HCM (HDBank)                                     | Ho Chi Minh City Development Joint Stock Commercial Bank | TP.HCM |
| TPB        | 970423  | Ngân hàng TMCP Tiên Phong (TPBank)                                            | Tien Phong Commercial Joint Stock Bank             | Hà Nội      |
| SCB        | 970429  | Ngân hàng TMCP Sài Gòn                                                        | Saigon Commercial Joint Stock Bank                 | TP.HCM      |
| OCB        | 970448  | Ngân hàng TMCP Phương Đông                                                    | Orient Commercial Joint Stock Bank                 | TP.HCM      |
| SHB        | 970443  | Ngân hàng TMCP Sài Gòn — Hà Nội                                               | Saigon-Hanoi Commercial Joint Stock Bank           | Hà Nội      |
| MSB        | 970426  | Ngân hàng TMCP Hàng Hải Việt Nam                                              | Vietnam Maritime Commercial Joint Stock Bank       | Hà Nội      |
| EIB        | 970431  | Ngân hàng TMCP Xuất Nhập khẩu Việt Nam (Eximbank)                             | Vietnam Export Import Commercial Joint Stock Bank  | TP.HCM      |
| LPB        | 970449  | Ngân hàng TMCP Bưu điện Liên Việt (LPBank)                                    | Lien Viet Post Joint Stock Commercial Bank         | Hà Nội      |
| SEAB       | 970440  | Ngân hàng TMCP Đông Nam Á (SeABank)                                           | Southeast Asia Commercial Joint Stock Bank         | Hà Nội      |
| VIB        | 970441  | Ngân hàng TMCP Quốc Tế Việt Nam                                               | Vietnam International Commercial Joint Stock Bank  | Hà Nội      |
| NAB        | 970428  | Ngân hàng TMCP Nam Á                                                          | Nam A Commercial Joint Stock Bank                  | TP.HCM      |

## Looking up BINs in code

Use `assets/bank-bins.json` for the machine-readable map (short code → BIN). The `parse_qr.py` script does the inverse lookup automatically.

## Coverage notes

- Foreign-bank Vietnam subsidiaries (HSBC, Standard Chartered, Shinhan, etc.) have BINs assigned by Napas; they are not bundled here because their VietQR participation varies and the public mapping changes. Add them per your operational need.
- Card-only schemes (Napas card BINs) start `9704` as well but are routed via `QRIBFTTC` not `QRIBFTTA`. This skill focuses on account-transfer (`QRIBFTTA`).

## Update policy

The BIN list is reviewed annually by Napas; renames and new banks (e.g. digital-bank rebrands) trigger BIN reassignments. Pin the skill version in production and bump on Napas notification.
