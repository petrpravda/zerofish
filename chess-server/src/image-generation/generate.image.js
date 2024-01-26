const { createCanvas, loadImage, registerFont } = require('canvas')
const fs = require('fs')

function wrapText(context, text, x, y, maxWidth, lineHeight) {
  var words = text.split(' ');
  var line = '';
  var lines = [];

  for (var i = 0; i < words.length; i++) {
    var testLine = line + words[i] + ' ';
    var metrics = context.measureText(testLine);
    var testWidth = metrics.width;

    if (testWidth > maxWidth && i > 0) {
      lines.push(line);
      line = words[i] + ' ';
    } else {
      line = testLine;
    }
  }

  lines.push(line);

  for (var j = 0; j < lines.length; j++) {
    context.fillText(lines[j], x, y + j * lineHeight);
  }
}

async function createImage() {
  const canvas = createCanvas(600, 320)
  const ctx = canvas.getContext('2d')


// Create a linear gradient
  const gradient = ctx.createLinearGradient(0, 0, canvas.width, canvas.height);
// Add color stops to the gradient
  gradient.addColorStop(0, 'hsla(167, 91%, 47%, 1)');
  gradient.addColorStop(1, 'hsla(207, 100%, 48%, 1)');
// Fill the canvas with the gradient
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  const image = await loadImage('logo.png');
  ctx.drawImage(image, 20, 20, 120, 120)



  registerFont('FiraSans-Black.ttf', { family: 'Fira Sans' })

// Display text in the middle of the canvas
  const text = 'on Scala';
  ctx.font = '72px "Fira Sans"'; // Set the font size and family
  ctx.fillStyle = 'white';
  // ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, 160, 80); // Draw the text

// // Draw line under text
// var text = ctx.measureText('Awesome!')
// ctx.lineTo(50 + text.width, 102)

  const title = 'Mastering Kotlin String Manipulation: From Basics to Efficient Substring Searching';
  ctx.font = '36px "Fira Sans"'; // Set the font size and family
  ctx.fillStyle = 'black';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  // ctx.fillText(title, canvas.width / 2, 240);
  wrapText(ctx, title, canvas.width / 2, 180, canvas.width - 20, 32);



  const out = fs.createWriteStream(__dirname + '/test.png')
  const stream = canvas.createPNGStream()
  stream.pipe(out)
  out.on('finish', () =>  console.log('The PNG file was created.'))
}

createImage().then();

