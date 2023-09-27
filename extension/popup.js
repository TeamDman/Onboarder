console.log("NewExtension.popup loaded");

// Initialize server URL input
const input = document.getElementById("server-url");
const defaultServerUrl = "https://127.0.0.1:5876";
input.setAttribute("placeholder", defaultServerUrl);

// Toast function
function toast(message) {
    const holder = document.getElementById("toast-holder");
    const toast = document.createElement("div");
    toast.className = "toast bg-green-500 text-white p-2 rounded-md mb-2 shadow-lg";
    toast.innerText = message;

    holder.appendChild(toast);
    setTimeout(() => {
        toast.remove();
    }, 3000);
}

// Server URL change event
input.addEventListener("change", function () {
    console.log("Server URL changed");
    const url = input.value;
    chrome.storage.local.set({ serverUrl: url }, function () {
        console.log("Server URL is set to " + url);
        toast("Server URL is set to " + url);
    });
});

// Load stored server URL
chrome.storage.local.get("serverUrl", function (data) {
    input.value = data.serverUrl || defaultServerUrl;
});

// Refresh button click event
document.getElementById("refresh-button").addEventListener("click", function () {
    const rand = Math.random().toString(36).substring(7);
    chrome.storage.local.set({ refresh: rand }, function () {
        console.log("Refresh is triggered");
        toast("Refresh is triggered");
    });
});
