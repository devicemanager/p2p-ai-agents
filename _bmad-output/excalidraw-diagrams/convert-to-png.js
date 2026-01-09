/**
 * Excalidraw to PNG Converter
 * Converts .excalidraw files to PNG using Puppeteer
 * 
 * Usage: node convert-to-png.js [input-file] [output-file]
 */

const fs = require('fs');
const path = require('path');

async function convertExcalidrawToPNG(inputFile, outputFile) {
  console.log('🎨 Excalidraw to PNG Converter');
  console.log('===============================\n');
  
  // Check if puppeteer is available
  let puppeteer;
  try {
    puppeteer = require('puppeteer');
  } catch (error) {
    console.error('❌ Puppeteer not installed!');
    console.log('\n📦 Install with: npm install puppeteer');
    console.log('\nAlternatively, use Method 1 (Web Interface):');
    console.log('1. Open https://excalidraw.com');
    console.log('2. Load your .excalidraw file');
    console.log('3. Export as PNG (scale 2x)');
    process.exit(1);
  }

  // Read the Excalidraw file
  console.log(`📂 Reading: ${inputFile}`);
  if (!fs.existsSync(inputFile)) {
    console.error(`❌ File not found: ${inputFile}`);
    process.exit(1);
  }

  const excalidrawData = JSON.parse(fs.readFileSync(inputFile, 'utf8'));
  console.log(`✅ Loaded ${excalidrawData.elements.length} elements\n`);

  // Launch browser
  console.log('🌐 Launching headless browser...');
  const browser = await puppeteer.launch({ headless: 'new' });
  const page = await browser.newPage();

  // Set viewport for high-quality render
  await page.setViewport({ width: 1400, height: 1200, deviceScaleFactor: 2 });

  // Load Excalidraw
  console.log('📥 Loading Excalidraw...');
  await page.goto('https://excalidraw.com', { waitUntil: 'networkidle2' });

  // Inject the scene data
  console.log('💉 Injecting diagram data...');
  await page.evaluate((data) => {
    const event = new CustomEvent('excalidraw-load', { 
      detail: { 
        elements: data.elements,
        appState: data.appState 
      } 
    });
    window.dispatchEvent(event);
  }, excalidrawData);

  // Wait for render
  await page.waitForTimeout(2000);

  // Take screenshot
  console.log('📸 Capturing screenshot...');
  const canvas = await page.$('.excalidraw__canvas');
  if (!canvas) {
    console.error('❌ Could not find Excalidraw canvas');
    await browser.close();
    process.exit(1);
  }

  await canvas.screenshot({ 
    path: outputFile,
    type: 'png'
  });

  await browser.close();

  // Verify output
  const stats = fs.statSync(outputFile);
  const sizeKB = (stats.size / 1024).toFixed(2);
  
  console.log(`\n✅ SUCCESS!`);
  console.log(`📁 Output: ${outputFile}`);
  console.log(`📊 Size: ${sizeKB} KB`);
  console.log(`\n🎉 Conversion complete!`);
}

// Main execution
const args = process.argv.slice(2);
const inputFile = args[0] || path.join(__dirname, 'diagram-expert-system-architecture.excalidraw');
const outputFile = args[1] || path.join(__dirname, 'diagram-expert-system-architecture.png');

convertExcalidrawToPNG(inputFile, outputFile)
  .catch(error => {
    console.error('\n❌ Error:', error.message);
    console.log('\n💡 Try Method 1 (Manual Export) instead:');
    console.log('   1. Open https://excalidraw.com');
    console.log('   2. Open → Select your .excalidraw file');
    console.log('   3. Export image → PNG → Scale 2x → Export');
    process.exit(1);
  });
