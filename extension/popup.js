// Function to remove an existing element by its ID
function removeElementById(id) {
    let element = document.getElementById(id);
    if (element) {
        element.parentNode.removeChild(element);
    }
}

// Function to add the text area and attach an event listener to it
function addTextArea(videoPlayerElement) {
    console.log("[Onboarder] adding content to page");

    // Unique ID for the text area
    const textAreaId = 'custom_notes_area';

    // Remove the existing text area if it exists
    removeElementById(textAreaId);

    // Create the text area element
    let textArea = document.createElement('textarea');

    // Assign a unique ID to the text area
    textArea.id = textAreaId;

    // Style the text area
    textArea.style.width = 'calc(100% - 35px)';
    textArea.style.padding = '10px';
    textArea.style.marginTop = '20px';
    textArea.style.marginLeft = '5px';
    textArea.style.borderRadius = '8px';
    textArea.style.backgroundColor = '#1f1f1f';
    textArea.style.color = '#ffffff';
    textArea.placeholder = 'notes';

    // Add a change event listener to send the content to the Rust endpoint
    textArea.addEventListener('input', function() {
        console.log("[Onboarder] Notes area modified. Current value:", this.value);
        // Create a POST request to the Rust HTTP server
        fetch('http://127.0.0.1:3000/invoke', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ content: this.value }),
        })
        .then(response => response.json())
        .then(data => {
            console.log('[Onboarder] Success:', data);
        })
        .catch((error) => {
            console.error('[Onboarder] Error:', error);
        });
    });

    // Insert the text area after the video player element
    videoPlayerElement.insertAdjacentElement('afterend', textArea);
}

console.log("[Onboarder] attaching load listener");

// Poll every 500 milliseconds until the video player element exists
let checkExist = setInterval(function() {
    let videoPlayerElement = document.getElementById('full-bleed-container');
    if (videoPlayerElement) {
        console.log("[Onboarder] video player element found");
        clearInterval(checkExist);
        addTextArea(videoPlayerElement);
    }
}, 500);
