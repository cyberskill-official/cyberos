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
const C: Record<string, Entry> = {
  // brand
  "brand.slogan": { en: "Turn Your Will Into Real", vi: "Hiện Thực Hoá Ý Chí" },

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
  "dash.mod.cuo.desc": { en: "Dream-loop envelope and FR backlog.", vi: "Vòng lặp dream-loop và backlog FR." },

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
  "header.searchTooltip": { en: "Search all channels (Ctrl/Cmd+K)", vi: "Tìm trong tất cả kênh (Ctrl/Cmd+K)" },
  "header.addPeople": { en: "Add people", vi: "Thêm người" },
  "header.searchPlaceholder": { en: "Search all channels", vi: "Tìm trong tất cả kênh" },
  "header.attachmentSnippet": { en: "[attachment]", vi: "[tệp đính kèm]" },

  // chat page
  "chat.welcomeTitle": { en: "Welcome to CyberOS Chat", vi: "Chào mừng bạn đến với CyberOS Chat" },
  "chat.welcomeSub": {
    en: "Pick a channel or start a direct message to begin.",
    vi: "Chọn một kênh hoặc mở tin nhắn trực tiếp để bắt đầu.",
  },
  "chat.noMessages": { en: "No messages yet. Say hello.", vi: "Chưa có tin nhắn nào. Gửi lời chào nhé." },
  "chat.jumpLatest": { en: "Viewing history · Jump to latest", vi: "Đang xem lịch sử · Về tin mới nhất" },
  "chat.newMessages": { en: "New messages", vi: "Tin nhắn mới" },
  "chat.typing": { en: "{name} is typing...", vi: "{name} đang nhập..." },
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
  "message.edit": { en: "Edit", vi: "Sửa" },
  "message.delete": { en: "Delete", vi: "Xóa" },
  "message.deleted": { en: "Message deleted", vi: "Đã xóa tin nhắn" },
  "message.undo": { en: "Undo", vi: "Hoàn tác" },

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
  "confirm.title": { en: "Please confirm", vi: "Vui lòng xác nhận" },

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
};

/// Translate a key, substituting {var} placeholders. Unknown keys render as the key itself (visible, safe).
export function t(key: string, vars?: Record<string, string | number>): string {
  const e = C[key];
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
