// GET LOG
window.addEventListener('DOMContentLoaded', () => {
    const logButton = document.getElementById('button_log');
    const resultDiv = document.getElementById('result_log');
    
    logButton.addEventListener('click', async () => {
        try {
            const response = await fetch('/api/get_log');
            const json = await response.json();
            resultDiv.textContent = json.data;
        } catch (error) {
            resultDiv.textContent = 'Error: ' + error;
        }
    });
});

// REBOOT GUEST
window.addEventListener('DOMContentLoaded', () => {
    const rebootButton = document.getElementById('button_reboot');
    const resultDiv = document.getElementById('result_reboot');
    
    rebootButton.addEventListener('click', async () => {
        try {
            const response = await fetch('/api/reboot_guest');
            const json = await response.json();
            resultDiv.textContent = json.data;
        } catch (error) {
            resultDiv.textContent = 'Error: ' + error;
        }
    });
});

// CONNECTION STATUS CHECK
window.addEventListener('DOMContentLoaded', () => {
    checkStatus();
    setInterval(checkStatus, 1000);
});

async function checkStatus() {
    const statusIndicator = document.getElementById('status_indicator');
    try {
        const response = await fetch('/api/health_check');
        const json = await response.json();
        if (json.data === "true") {
            statusIndicator.classList.remove('red');
            statusIndicator.classList.add('green');
        } else {
            statusIndicator.classList.remove('green');
            statusIndicator.classList.add('red');
        }
    } catch (error) {
        statusIndicator.classList.remove('green');
        statusIndicator.classList.add('red');
    }
}
