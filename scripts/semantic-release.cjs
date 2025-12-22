const { writeFileSync } = require('fs');
const { resolve } = require('path');

async function run() {
  const { default: semanticRelease } = await require('semantic-release');
  
  const result = await semanticRelease();

  if (result && result.nextRelease) {
    const releaseInfo = {
      version: result.nextRelease.version,
      gitTag: result.nextRelease.gitTag,
    };
    writeFileSync(resolve(process.cwd(), 'release.json'), JSON.stringify(releaseInfo));
  }
}

run().catch(error => {
  console.error('The automated release failed:', error);
  process.exit(1);
});
