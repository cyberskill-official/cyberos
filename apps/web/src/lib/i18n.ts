// Bilingual UI (remaining-gaps wave): a hand-written EN/VI catalog with a tiny t() - no i18n framework for
// a two-language internal tool. The language is chosen once per page load (stored choice, else the browser
// language), and switching persists + reloads, so every component reads t() as a plain function with zero
// re-render plumbing. Keys are dotted by surface ("chat.noMessages"); an unknown key renders as itself so a
// missed entry is visible, never a crash. Vietnamese entries use real diacritics and the team's tone (thân
// thiện, ngắn gọn).

export type Lang = "en" | "vi";

const KEY = "cyberos.lang";

export function storedLang(): Lang | null {
  try {
    const v = localStorage.getItem(KEY);
    return v === "en" || v === "vi" ? v : null;
  } catch {
    return null;
  }
}

export function currentLang(): Lang {
  const stored = storedLang();
  if (stored) return stored;
  try {
    return (navigator.language || "").toLowerCase().startsWith("vi") ? "vi" : "en";
  } catch {
    return "en";
  }
}

/// Persist the choice and reload - the whole UI re-renders in the new language on the fresh load.
export function setLang(l: Lang) {
  try {
    localStorage.setItem(KEY, l);
  } catch {
    /* private mode: the switch still applies for this load via reload + browser language */
  }
  location.reload();
}

const LANG: Lang = currentLang();

type Entry = { en: string; vi: string };

// The catalog. Grouped by surface; keep keys stable once shipped (they are the contract between components
// and translations). {var} placeholders are substituted by t().
const C = {
  // brand
  "brand.slogan": { en: "Turn Your Will Into Real", vi: "Hiện Thực Hoá Ý Chí" },

  // update prompt (a new build was deployed)
  "update.available": { en: "A new version of CyberOS is available.", vi: "Đã có phiên bản CyberOS mới." },
  "update.reload": { en: "Reload", vi: "Tải lại" },

  // shared
  "common.cancel": { en: "Cancel", vi: "Hủy" },
  "common.save": { en: "Save", vi: "Lưu" },
  "common.close": { en: "Close", vi: "Đóng" },
  "common.create": { en: "Create", vi: "Tạo" },
  "common.add": { en: "Add", vi: "Thêm" },
  "common.send": { en: "Send", vi: "Gửi" },
  "common.search": { en: "Search", vi: "Tìm kiếm" },
  "common.loading": { en: "Loading...", vi: "Đang tải..." },
  "common.refresh": { en: "Refresh", vi: "Làm mới" },
  "common.open": { en: "Open", vi: "Mở" },
  "common.join": { en: "Join", vi: "Tham gia" },
  "common.you": { en: "You", vi: "Bạn" },

  // quick switcher (Cmd/Ctrl+K)
  "switch.title": { en: "Jump to", vi: "Chuyển nhanh" },
  "switch.placeholder": {
    en: "Jump to a channel, DM, or person...",
    vi: "Chuyển đến kênh, tin nhắn, hoặc người...",
  },
  "switch.noMatch": { en: "No matches", vi: "Không có kết quả" },
  "switch.dmHint": { en: "Message", vi: "Nhắn tin" },

  // topbar
  "top.allModules": { en: "All modules", vi: "Tất cả mô-đun" },
  "top.backToChat": { en: "Back to chat", vi: "Quay lại chat" },
  "top.docs": { en: "Docs", vi: "Tài liệu" },
  "version.current": { en: "CyberOS version", vi: "Phiên bản CyberOS" },
  "module.stub": {
    en: "This module's app surface ships here. Its manual and guides are already on the docs site.",
    vi: "Giao diện của mô-đun này sẽ xuất hiện tại đây. Tài liệu và hướng dẫn đã có trên trang tài liệu.",
  },
  "module.manual": { en: "Open the manual", vi: "Mở tài liệu" },
  "top.signOut": { en: "Sign out", vi: "Đăng xuất" },
  "top.themeToLight": { en: "Switch to light theme", vi: "Chuyển sang giao diện sáng" },
  "top.themeToDark": { en: "Switch to dark theme", vi: "Chuyển sang giao diện tối" },
  "top.language": { en: "Tiếng Việt", vi: "English" },

  // login
  "login.signIn": { en: "Sign in", vi: "Đăng nhập" },
  "login.subtitle": { en: "to your CyberOS workspace", vi: "vào không gian làm việc CyberOS của bạn" },
  "login.google": { en: "Sign in with Google", vi: "Đăng nhập bằng Google" },
  "login.adminSignIn": { en: "Admin sign-in", vi: "Đăng nhập admin" },
  "login.workspace": { en: "Workspace (tenant)", vi: "Không gian làm việc (tenant)" },
  "login.handle": { en: "Handle", vi: "Handle" },
  "login.password": { en: "Password", vi: "Mật khẩu" },
  "login.signingIn": { en: "Signing in...", vi: "Đang đăng nhập..." },

  // dashboard
  "dash.title": { en: "Workspace", vi: "Không gian làm việc" },
  "dash.hint": { en: "Pick a module to begin.", vi: "Chọn một mô-đun để bắt đầu." },
  "dash.soon": { en: "Soon", vi: "Sắp có" },
  "dash.mod.chat.name": { en: "Chat", vi: "Chat" },
  "dash.mod.chat.desc": { en: "Channels, messages, and live presence.", vi: "Kênh, tin nhắn và trạng thái trực tuyến." },
  "dash.mod.assistant.name": { en: "Assistant", vi: "Trợ lý" },
  "dash.mod.assistant.desc": { en: "Talk to your model via the AI gateway.", vi: "Trò chuyện với mô hình qua AI gateway." },
  "dash.mod.ai.name": { en: "AI Ops", vi: "AI Ops" },
  "dash.mod.ai.desc": {
    en: "Provider routing, spend caps, residency.",
    vi: "Định tuyến nhà cung cấp, hạn mức chi tiêu, nơi lưu dữ liệu.",
  },
  "dash.mod.mcp.name": { en: "MCP Registry", vi: "MCP Registry" },
  "dash.mod.mcp.desc": { en: "Tools the MCP gateway is serving.", vi: "Các công cụ MCP gateway đang phục vụ." },
  "dash.mod.memory.name": { en: "Memory & Audit", vi: "Bộ nhớ & Audit" },
  "dash.mod.memory.desc": { en: "The tenant's hash-chained audit log.", vi: "Nhật ký audit chuỗi băm của tenant." },
  "dash.mod.cuo.name": { en: "Workflows & GENIE", vi: "Quy trình & GENIE" },
  "dash.mod.cuo.desc": { en: "Dream-loop envelope and task backlog.", vi: "Vòng lặp dream-loop và backlog task." },

  // sidebar
  "sidebar.editProfile": { en: "Edit your profile", vi: "Chỉnh sửa hồ sơ của bạn" },
  "sidebar.channels": { en: "Channels", vi: "Kênh" },
  "sidebar.browseChannels": { en: "Browse public channels", vi: "Duyệt kênh công khai" },
  "sidebar.newChannel": { en: "New channel", vi: "Tạo kênh mới" },
  "sidebar.noChannels": { en: "No channels yet", vi: "Chưa có kênh nào" },
  "sidebar.archived": { en: "Archived", vi: "Đã lưu trữ" },
  "sidebar.dms": { en: "Direct messages", vi: "Tin nhắn trực tiếp" },
  "sidebar.newDm": { en: "New direct message", vi: "Tin nhắn trực tiếp mới" },
  "sidebar.noDms": { en: "No direct messages", vi: "Chưa có tin nhắn trực tiếp" },
  "sidebar.mentionCount_one": { en: "1 mention", vi: "1 lượt nhắc" },
  "sidebar.mentionCount_other": { en: "{n} mentions", vi: "{n} lượt nhắc" },
  "sidebar.connected": { en: "Connected", vi: "Đã kết nối" },
  "sidebar.reconnecting": { en: "Reconnecting...", vi: "Đang kết nối lại..." },
  "sidebar.connecting": { en: "Connecting...", vi: "Đang kết nối..." },

  // channel header
  "header.aiTooltip": {
    en: "AI assistant: catch me up, action items",
    vi: "Trợ lý AI: tóm tắt hội thoại, việc cần làm",
  },
  "header.voiceCall": { en: "Voice call", vi: "Gọi thoại" },
  "header.videoCall": { en: "Video call", vi: "Gọi video" },
  "header.searchTooltip": { en: "Search all channels", vi: "Tìm trong tất cả kênh" },
  "header.mute": { en: "Mute notifications", vi: "Tắt thông báo" },
  "header.unmute": { en: "Unmute notifications", vi: "Bật thông báo" },
  "header.addPeople": { en: "Add people", vi: "Thêm người" },
  "header.searchPlaceholder": { en: "Search all channels", vi: "Tìm trong tất cả kênh" },
  "header.attachmentSnippet": { en: "[attachment]", vi: "[tệp đính kèm]" },
  "header.searchCount": { en: "{n} results", vi: "{n} kết quả" },

  // chat page
  "chat.welcomeTitle": { en: "Welcome to CyberOS Chat", vi: "Chào mừng bạn đến với CyberOS Chat" },
  "chat.welcomeSub": {
    en: "Pick a channel or start a direct message to begin.",
    vi: "Chọn một kênh hoặc mở tin nhắn trực tiếp để bắt đầu.",
  },
  "chat.noMessages": { en: "No messages yet. Say hello.", vi: "Chưa có tin nhắn nào. Gửi lời chào nhé." },
  "chat.jumpLatest": { en: "Viewing history · Jump to latest", vi: "Đang xem lịch sử · Về tin mới nhất" },
  "chat.newMessages": { en: "New messages", vi: "Tin nhắn mới" },
  "chat.typing": { en: "{name} is typing", vi: "{name} đang nhập" },
  "chat.dropToShare": { en: "Drop to share", vi: "Thả để chia sẻ" },
  "chat.seen": { en: "Seen", vi: "Đã xem" },
  "chat.seenBy": { en: "Seen by {names}", vi: "{names} đã xem" },
  "chat.seenByCount": { en: "Seen by {n}", vi: "{n} người đã xem" },
  "chat.activeNow": { en: "Active now", vi: "Đang hoạt động" },
  "chat.directMessage": { en: "Direct message", vi: "Tin nhắn trực tiếp" },
  "chat.onlineCount": { en: "{n} online", vi: "{n} đang trực tuyến" },
  "chat.channel": { en: "Channel", vi: "Kênh" },
  "chat.notifyTitleIn": { en: "{who} in {channel}", vi: "{who} trong {channel}" },
  "chat.notifyMention": { en: "mentioned you: {preview}", vi: "đã nhắc đến bạn: {preview}" },
  "chat.notifyNew": { en: "New message", vi: "Tin nhắn mới" },
  "chat.aiSuggestUnavailable": {
    en: "AI suggestions are unavailable right now (the AI gateway is not serving).",
    vi: "Gợi ý AI hiện chưa dùng được (AI gateway chưa hoạt động).",
  },
  "chat.attachTooMany": { en: "At most {limit} files per message.", vi: "Tối đa {limit} tệp mỗi tin nhắn." },
  "chat.attachTooBig": {
    en: '"{name}" is {size}, over the {limit} limit.',
    vi: '"{name}" nặng {size}, vượt giới hạn {limit}.',
  },
  "chat.dismissSuggestions": { en: "Dismiss suggestions", vi: "Ẩn gợi ý" },
  "chat.archivedNote": {
    en: "This channel is archived and read-only.",
    vi: "Kênh này đã được lưu trữ và chỉ đọc.",
  },

  // composer
  "composer.placeholder": { en: "Message {name}", vi: "Nhắn cho {name}" },
  "composer.placeholderFile": {
    en: "Add a message or just send the files",
    vi: "Thêm lời nhắn hoặc gửi luôn tệp",
  },
  "composer.attachFile": { en: "Attach a file", vi: "Đính kèm tệp" },
  "composer.emoji": { en: "Emoji", vi: "Emoji" },
  "composer.suggestReplies": { en: "Suggest replies (AI)", vi: "Gợi ý trả lời (AI)" },
  "composer.removeAttachment": { en: "Remove attachment", vi: "Gỡ tệp đính kèm" },
  "composer.hint": {
    en: "Enter to send · Shift+Enter for a new line",
    vi: "Enter để gửi · Shift+Enter để xuống dòng",
  },
  "composer.user": { en: "user", vi: "người dùng" },

  // message row
  "message.edited": { en: "edited", vi: "đã sửa" },
  "message.sending": { en: "Sending...", vi: "Đang gửi..." },
  "message.failed": { en: "Not sent", vi: "Chưa gửi được" },
  "message.retry": { en: "Retry", vi: "Thử lại" },
  "message.addReaction": { en: "Add reaction", vi: "Thêm cảm xúc" },
  "message.removeReaction": { en: "Remove your reaction", vi: "Gỡ cảm xúc của bạn" },
  "message.react": { en: "React", vi: "Bày tỏ cảm xúc" },
  "message.allEmoji": { en: "All emoji", vi: "Tất cả emoji" },
  "message.translate": { en: "Translate to English", vi: "Dịch sang tiếng Anh" },
  "message.translating": { en: "Translating...", vi: "Đang dịch..." },
  "message.translationLabel": { en: "English", vi: "Tiếng Anh" },
  "message.translateUnavailable": { en: "Translation unavailable", vi: "Tính năng dịch hiện chưa dùng được" },
  "message.replyInThread": { en: "Reply in thread", vi: "Trả lời theo luồng" },
  "message.replyOne": { en: "{n} reply", vi: "{n} phản hồi" },
  "message.replyMany": { en: "{n} replies", vi: "{n} phản hồi" },
  "message.edit": { en: "Edit", vi: "Sửa" },
  "message.delete": { en: "Delete", vi: "Xóa" },
  "message.deleted": { en: "Message deleted", vi: "Đã xóa tin nhắn" },
  "message.undo": { en: "Undo", vi: "Hoàn tác" },
  "message.editHint": {
    en: "Enter to save · Shift+Enter for a new line · Esc to cancel",
    vi: "Enter để lưu · Shift+Enter để xuống dòng · Esc để hủy",
  },

  // thread panel
  "thread.title": { en: "Thread", vi: "Luồng" },
  "thread.close": { en: "Close thread", vi: "Đóng luồng" },
  "thread.replyCount_one": { en: "1 reply", vi: "1 trả lời" },
  "thread.replyCount_other": { en: "{n} replies", vi: "{n} trả lời" },
  "thread.replyPlaceholder": { en: "Reply...", vi: "Trả lời..." },
  "thread.reply": { en: "Reply", vi: "Trả lời" },

  // AI panel
  "ai.title": { en: "Assistant", vi: "Trợ lý" },
  "ai.catchMeUp": { en: "Catch me up", vi: "Tóm tắt giúp tôi" },
  "ai.actionItems": { en: "Action items", vi: "Việc cần làm" },
  "ai.thinking": { en: "Thinking...", vi: "Đang suy nghĩ..." },
  "ai.meta": {
    en: "Based on the last {n} messages · AI-generated, double-check.",
    vi: "Dựa trên {n} tin nhắn gần nhất · Nội dung do AI tạo, hãy kiểm tra lại.",
  },
  "ai.unavailable": {
    en: "AI is unavailable right now (the AI gateway is not serving). Chat itself is unaffected.",
    vi: "AI hiện chưa dùng được (AI gateway chưa hoạt động). Chat vẫn hoạt động bình thường.",
  },

  // channel settings
  "settings.title": { en: "Channel settings", vi: "Cài đặt kênh" },
  "settings.archivedNote": { en: "This channel is archived (read-only).", vi: "Kênh này đã được lưu trữ (chỉ đọc)." },
  "settings.name": { en: "Name", vi: "Tên kênh" },
  "settings.topic": { en: "Topic", vi: "Chủ đề" },
  "settings.topicPlaceholder": { en: "What is this channel for?", vi: "Kênh này dùng để làm gì?" },
  "settings.visibility": { en: "Visibility", vi: "Chế độ hiển thị" },
  "settings.private": { en: "Private", vi: "Riêng tư" },
  "settings.privateSub": { en: "Members only; joined by invite", vi: "Chỉ thành viên; tham gia qua lời mời" },
  "settings.public": { en: "Public", vi: "Công khai" },
  "settings.publicSub": {
    en: "Anyone on the team can browse + join",
    vi: "Mọi người trong nhóm đều có thể xem và tham gia",
  },
  "settings.notify": { en: "Notify me about", vi: "Thông báo cho tôi về" },
  "settings.notifyAll": { en: "Every message", vi: "Mọi tin nhắn" },
  "settings.notifyMentions": { en: "Only @-mentions", vi: "Chỉ khi được @-nhắc" },
  "settings.notifyNone": { en: "Nothing (mute)", vi: "Không thông báo (tắt tiếng)" },
  "settings.members": { en: "Members ({n})", vi: "Thành viên ({n})" },
  "settings.youSuffix": { en: " (you)", vi: " (bạn)" },
  "settings.removeFromChannel": { en: "Remove from channel", vi: "Xóa khỏi kênh" },
  "settings.leaveChannel": { en: "Leave channel", vi: "Rời kênh" },
  "settings.archive": { en: "Archive", vi: "Lưu trữ" },
  "settings.unarchive": { en: "Unarchive", vi: "Bỏ lưu trữ" },
  "settings.confirmArchive": {
    en: "Archive this channel? It becomes read-only and drops out of the channel browser.",
    vi: "Lưu trữ kênh này? Kênh sẽ chuyển sang chỉ đọc và không còn xuất hiện trong danh sách duyệt kênh.",
  },
  "settings.confirmUnarchive": { en: "Unarchive this channel?", vi: "Bỏ lưu trữ kênh này?" },
  "settings.confirmRemove": { en: "Remove {name} from {channel}?", vi: "Xóa {name} khỏi {channel}?" },
  "settings.confirmLeave": { en: "Leave {channel}?", vi: "Rời {channel}?" },
  "settings.thisChannel": { en: "this channel", vi: "kênh này" },
  "role.owner": { en: "Owner", vi: "Chủ kênh" },
  "role.admin": { en: "Admin", vi: "Quản trị" },
  "role.member": { en: "Member", vi: "Thành viên" },
  // report (TASK-CHAT-267). Every string the report dialog renders ships in both locales: a Vietnamese-speaking
  // employee reporting harassment in English is a failure of the product, not of the employee (§1 #12).
  "report.title": { en: "Report", vi: "Báo cáo" },
  "report.subtitleMessage": {
    en: "Report this message to your workspace administrator.",
    vi: "Báo cáo tin nhắn này tới quản trị viên không gian làm việc.",
  },
  "report.subtitleAttachment": {
    en: "Report this file to your workspace administrator.",
    vi: "Báo cáo tệp này tới quản trị viên không gian làm việc.",
  },
  "report.subtitleSubject": {
    en: "Report this person to your workspace administrator.",
    vi: "Báo cáo người này tới quản trị viên không gian làm việc.",
  },
  "report.reasonLegend": { en: "Why are you reporting this?", vi: "Vì sao bạn báo cáo?" },
  "report.reason.harassment": { en: "Harassment or bullying", vi: "Quấy rối hoặc bắt nạt" },
  "report.reason.hate": { en: "Hate speech", vi: "Ngôn từ thù ghét" },
  "report.reason.sexual": { en: "Sexual content", vi: "Nội dung tình dục" },
  "report.reason.violence": { en: "Violence or threats", vi: "Bạo lực hoặc đe dọa" },
  "report.reason.self_harm": { en: "Self-harm or suicide", vi: "Tự làm hại bản thân hoặc tự tử" },
  "report.reason.illegal": { en: "Illegal activity", vi: "Hoạt động bất hợp pháp" },
  "report.reason.spam": { en: "Spam", vi: "Spam" },
  "report.reason.other": { en: "Something else", vi: "Lý do khác" },
  "report.detailLabel": { en: "Anything else? (optional)", vi: "Bạn muốn nói thêm? (không bắt buộc)" },
  "report.detailPlaceholder": {
    en: "Add context that would help an administrator.",
    vi: "Thêm bối cảnh giúp quản trị viên hiểu rõ hơn.",
  },
  "report.submit": { en: "Send report", vi: "Gửi báo cáo" },
  "report.submitting": { en: "Sending…", vi: "Đang gửi…" },
  "report.sent": { en: "Report sent. Thank you.", vi: "Đã gửi báo cáo. Cảm ơn bạn." },
  "report.failed": { en: "Could not send the report. Try again.", vi: "Không gửi được báo cáo. Vui lòng thử lại." },
  // The reporter is told, plainly, that the reported person is not notified (§1 #5). People do not report
  // harassment if they think the person will find out.
  "report.privacyNote": {
    en: "Your report is private. The person you report is not told who reported them.",
    vi: "Báo cáo của bạn được giữ kín. Người bị báo cáo không biết ai đã báo cáo họ.",
  },
  "report.action": { en: "Report", vi: "Báo cáo" },
  "report.reportPerson": { en: "Report this person", vi: "Báo cáo người này" },

  // moderation (TASK-CHAT-269)
  "mod.title": { en: "Moderation", vi: "Kiểm duyệt" },
  "mod.empty": { en: "Nothing to review.", vi: "Không có gì cần xem xét." },
  "mod.reportCount": { en: "{n} report(s)", vi: "{n} báo cáo" },
  "mod.reportedBy": { en: "Reported by {name}", vi: "Được báo cáo bởi {name}" },
  "mod.evidence": { en: "What was reported", vi: "Nội dung bị báo cáo" },
  // §1 #7 — the difference between these two sentences IS evidence.
  "mod.originalPresent": { en: "The original is still posted.", vi: "Nội dung gốc vẫn còn." },
  "mod.originalGone": {
    en: "The sender has since removed the original.",
    vi: "Người gửi đã xóa nội dung gốc sau đó.",
  },
  "mod.context": { en: "Context", vi: "Bối cảnh" },
  "mod.noContext": { en: "No surrounding context.", vi: "Không có bối cảnh xung quanh." },
  // Say WHY, plainly. A silent empty panel invites someone to "fix" it by fetching the DM thread — the one
  // thing this task exists to prevent (§1 #9).
  "mod.noContextDm": {
    en: "Only the reported message is shown. Direct messages are not disclosed, and a private channel you are not in is not shown.",
    vi: "Chỉ hiển thị tin nhắn bị báo cáo. Tin nhắn riêng không được tiết lộ, và kênh riêng tư bạn không tham gia sẽ không hiện ra.",
  },
  "mod.note": { en: "Note (optional)", vi: "Ghi chú (không bắt buộc)" },
  "mod.notePlaceholder": { en: "Why you decided this.", vi: "Lý do bạn quyết định như vậy." },
  "mod.dismiss": { en: "Dismiss", vi: "Bỏ qua" },
  "mod.deleteMessage": { en: "Delete message", vi: "Xóa tin nhắn" },
  // §1 #14 — the label says CHANNEL, because that is what it does. A button that read "Remove member" and
  // removed someone from the organisation would be a very expensive ambiguity.
  "mod.removeMember": { en: "Remove from channel", vi: "Xóa khỏi kênh" },
  "mod.resolved": { en: "Resolved.", vi: "Đã xử lý." },
  "mod.failed": { en: "Something went wrong. Try again.", vi: "Đã có lỗi. Vui lòng thử lại." },
  "mod.contentPolicy": { en: "Content policy", vi: "Chính sách nội dung" },
  "mod.severity": { en: "Severity", vi: "Mức độ" },
  "top.moderation": { en: "Moderation", vi: "Kiểm duyệt" },

  // blocking (TASK-CHAT-268)
  "blocked.hidden": {
    en: "Message from {name}, who you blocked.",
    vi: "Tin nhắn từ {name}, người bạn đã chặn.",
  },
  "blocked.revealed": {
    en: "{name} posted here. The content was not delivered to you.",
    vi: "{name} đã đăng ở đây. Nội dung không được gửi tới bạn.",
  },
  "blocked.showAnyway": { en: "Show anyway", vi: "Vẫn hiện" },
  "blocked.block": { en: "Block", vi: "Chặn" },
  "blocked.unblock": { en: "Unblock", vi: "Bỏ chặn" },
  "blocked.blockPerson": { en: "Block this person", vi: "Chặn người này" },
  "blocked.unblockPerson": { en: "Unblock this person", vi: "Bỏ chặn người này" },
  // The confirm copy states plainly what a block does AND what it does not do. "They are not told" is the
  // sentence that decides whether someone actually uses the feature (§2).
  "blocked.confirmBlock": {
    en: "Block {name}? You will stop seeing their messages, and they will not be told. You can undo this at any time.",
    vi: "Chặn {name}? Bạn sẽ không thấy tin nhắn của họ nữa, và họ không được thông báo. Bạn có thể bỏ chặn bất cứ lúc nào.",
  },
  "blocked.confirmUnblock": {
    en: "Unblock {name}? Their messages, including any sent while blocked, will become visible again.",
    vi: "Bỏ chặn {name}? Tin nhắn của họ, kể cả tin đã gửi trong lúc bị chặn, sẽ hiện lại.",
  },
  "blocked.failed": { en: "Could not update the block. Try again.", vi: "Không cập nhật được. Vui lòng thử lại." },

  "confirm.title": { en: "Please confirm", vi: "Vui lòng xác nhận" },
  "sheet.title": { en: "Message actions", vi: "Tùy chọn tin nhắn" },
  "a11y.channelsNav": { en: "Channels and direct messages", vi: "Kênh và tin nhắn trực tiếp" },
  "a11y.unread": { en: "{n} unread", vi: "{n} chưa đọc" },
  "a11y.mentions": { en: "{n} mentions", vi: "{n} lượt nhắc" },
  "a11y.muted": { en: "muted", vi: "đã tắt tiếng" },
  "a11y.attachment": { en: "sent an attachment", vi: "đã gửi tệp đính kèm" },

  // browse channels
  "browse.title": { en: "Browse channels", vi: "Duyệt kênh" },
  "browse.filter": { en: "Filter channels", vi: "Lọc kênh" },
  "browse.noPublic": { en: "No public channels yet", vi: "Chưa có kênh công khai nào" },
  "browse.noMatch": { en: "No channels match", vi: "Không có kênh nào khớp" },
  "browse.memberCount_one": { en: "1 member", vi: "1 thành viên" },
  "browse.memberCount_other": { en: "{n} members", vi: "{n} thành viên" },

  // people picker
  "picker.startCall": { en: "Start a call", vi: "Bắt đầu cuộc gọi" },
  "picker.newGroup": { en: "New group channel", vi: "Tạo kênh nhóm mới" },
  "picker.channelName": { en: "Channel name", vi: "Tên kênh" },
  "picker.searchTeammates": { en: "Search teammates", vi: "Tìm đồng nghiệp" },
  "picker.noMatch": { en: "No teammates match", vi: "Không tìm thấy đồng nghiệp nào khớp" },
  "picker.directoryUnavailable": { en: "Directory unavailable", vi: "Danh bạ hiện chưa dùng được" },

  // profile editor
  "profile.title": { en: "Edit profile", vi: "Chỉnh sửa hồ sơ" },
  "profile.uploadPhoto": { en: "Upload photo", vi: "Tải ảnh lên" },
  "profile.remove": { en: "Remove", vi: "Gỡ ảnh" },
  "profile.displayName": { en: "Display name", vi: "Tên hiển thị" },
  "profile.saving": { en: "Saving...", vi: "Đang lưu..." },
  "profile.readError": { en: "Could not read that image.", vi: "Không đọc được ảnh này." },
  "profile.nameRequired": { en: "Display name cannot be empty.", vi: "Tên hiển thị không được để trống." },

  // lightbox
  "lightbox.download": { en: "Download", vi: "Tải xuống" },

  // calls
  "call.incomingVoice": { en: "Incoming voice call", vi: "Cuộc gọi thoại đến" },
  "call.incomingVideo": { en: "Incoming video call", vi: "Cuộc gọi video đến" },
  "call.accept": { en: "Accept", vi: "Nghe máy" },
  "call.decline": { en: "Decline", vi: "Từ chối" },
  "call.calling": { en: "Calling...", vi: "Đang gọi..." },
  "call.connecting": { en: "Connecting...", vi: "Đang kết nối..." },
  "call.mute": { en: "Mute", vi: "Tắt mic" },
  "call.unmute": { en: "Unmute", vi: "Bật mic" },
  "call.camOn": { en: "Turn camera on", vi: "Bật camera" },
  "call.camOff": { en: "Turn camera off", vi: "Tắt camera" },
  "call.hangup": { en: "Hang up", vi: "Kết thúc cuộc gọi" },

  // attachments
  "attachment.unavailable": { en: "attachment unavailable", vi: "tệp đính kèm không khả dụng" },
  "attachment.loading": { en: "loading...", vi: "đang tải..." },

  // emoji picker
  "emoji.search": { en: "Search emoji", vi: "Tìm emoji" },
  "emoji.skinTone": { en: "Skin tone", vi: "Tông màu da" },
  "emoji.loading": { en: "Loading emoji...", vi: "Đang tải emoji..." },
  "emoji.loadFailed": { en: "Emoji data failed to load.", vi: "Không tải được dữ liệu emoji." },
  "emoji.noMatch": { en: 'No emoji match "{q}"', vi: 'Không có emoji nào khớp "{q}"' },
  "emoji.frequent": { en: "Frequently used", vi: "Hay dùng" },
} satisfies Record<string, Entry>;

/// Translate a key, substituting {var} placeholders. Unknown keys render as the key itself (visible, safe).
// Every key the catalog actually defines. Exported so a surface can prove, at COMPILE time, that each string
// it renders exists — see REPORT_DIALOG_KEYS in components/ReportDialog.tsx, which is declared
// `satisfies readonly CatalogKey[]`. Because an Entry is `{ en; vi }`, a key that type-checks necessarily
// carries BOTH locales: TASK-CHAT-267 §1 #12 is then enforced by `tsc --noEmit`, not by a convention someone
// has to remember. A missing key is a build failure, not a string that silently renders as its own name.
export type CatalogKey = keyof typeof C;

export function t(key: string, vars?: Record<string, string | number>): string {
  // `t` stays permissive on purpose: it takes any string and renders an unknown key as itself, so a missed
  // entry shows up in the UI rather than crashing. The cast is what lets that coexist with C's INFERRED
  // literal keys — and those literal keys are the whole point, because they are what makes CatalogKey a real
  // compile-time check. Re-annotating C as `Record<string, Entry>` would restore the index signature and
  // silently destroy that check (it renders CatalogKey = string, and nothing fails). Verified by removing a
  // key and watching tsc go red; do not "simplify" this back.
  const e = (C as Record<string, Entry>)[key];
  let s = e ? e[LANG] : key;
  if (vars) {
    for (const [k, v] of Object.entries(vars)) s = s.split(`{${k}}`).join(String(v));
  }
  return s;
}

/// Extend the catalog (used only by the catalog file itself as it grows; components never call this).
export function define(entries: Record<string, Entry>) {
  Object.assign(C, entries);
}
