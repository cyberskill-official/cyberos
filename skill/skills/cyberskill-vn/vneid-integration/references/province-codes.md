# Vietnamese province codes (GSO 3-digit, used in CCCD digits 1–3)

These are the 63 currently-active provinces and centrally-controlled cities of Vietnam, with their 3-digit codes per General Statistics Office (GSO) standard. The codes also appear as CCCD digits 1–3 and as Mã tỉnh in various government IT systems.

Gaps in the numeric range (e.g. `003`, `005`, `007`, `009`, …) are historical retirements — provinces that were merged, renamed, or absorbed in past administrative restructurings.

## Northern Vietnam

| Code | Province / city       |
|------|-----------------------|
| 001  | Hà Nội                |
| 002  | Hà Giang              |
| 004  | Cao Bằng              |
| 006  | Bắc Kạn               |
| 008  | Tuyên Quang           |
| 010  | Lào Cai               |
| 011  | Điện Biên             |
| 012  | Lai Châu              |
| 014  | Sơn La                |
| 015  | Yên Bái               |
| 017  | Hoà Bình              |
| 019  | Thái Nguyên           |
| 020  | Lạng Sơn              |
| 022  | Quảng Ninh            |
| 024  | Bắc Giang             |
| 025  | Phú Thọ               |
| 026  | Vĩnh Phúc             |
| 027  | Bắc Ninh              |
| 030  | Hải Dương             |
| 031  | Hải Phòng             |
| 033  | Hưng Yên              |
| 034  | Thái Bình             |
| 035  | Hà Nam                |
| 036  | Nam Định              |
| 037  | Ninh Bình             |

## North-Central / Central Vietnam

| Code | Province / city       |
|------|-----------------------|
| 038  | Thanh Hóa             |
| 040  | Nghệ An               |
| 042  | Hà Tĩnh               |
| 044  | Quảng Bình            |
| 045  | Quảng Trị             |
| 046  | Thừa Thiên Huế        |
| 048  | Đà Nẵng               |
| 049  | Quảng Nam             |
| 051  | Quảng Ngãi            |
| 052  | Bình Định             |
| 054  | Phú Yên               |
| 056  | Khánh Hòa             |
| 058  | Ninh Thuận            |
| 060  | Bình Thuận            |

## Central Highlands

| Code | Province / city       |
|------|-----------------------|
| 062  | Kon Tum               |
| 064  | Gia Lai               |
| 066  | Đắk Lắk               |
| 067  | Đắk Nông              |
| 068  | Lâm Đồng              |

## Southern Vietnam

| Code | Province / city       |
|------|-----------------------|
| 070  | Bình Phước            |
| 072  | Tây Ninh              |
| 074  | Bình Dương            |
| 075  | Đồng Nai              |
| 077  | Bà Rịa - Vũng Tàu     |
| 079  | Hồ Chí Minh           |
| 080  | Long An               |
| 082  | Tiền Giang            |
| 083  | Bến Tre               |
| 084  | Trà Vinh              |
| 086  | Vĩnh Long             |
| 087  | Đồng Tháp             |
| 089  | An Giang              |
| 091  | Kiên Giang            |
| 092  | Cần Thơ               |
| 093  | Hậu Giang             |
| 094  | Sóc Trăng             |
| 095  | Bạc Liêu              |
| 096  | Cà Mau                |

## Why CCCDs are tied to province of registration, not residence

A citizen's CCCD encodes the **province of household registration (hộ khẩu)** at the time of issuance — not their current residence. This sometimes confuses foreign-partner KYC pipelines that compare CCCD-province against an address-province and reject mismatches. They shouldn't — Vietnamese citizens frequently relocate without re-registering, and the CCCD number never changes.

## Update policy

The GSO updates province codes when administrative units merge or split. The next major realignment is anticipated mid-2025; the schedule is set by the National Assembly's Resolution on Administrative Restructuring. This skill should be re-validated after each such event.
