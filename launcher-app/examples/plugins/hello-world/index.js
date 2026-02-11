#!/usr/bin/env node
/**
 * Hello World Plugin — minimal example for the Launcher plugin system.
 *
 * Protocol:
 *   - Reads JSON from stdin: { command, query, preferences }
 *   - Writes JSON to stdout: { items: [...] }
 */

// Read full stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => { input += chunk; });
process.stdin.on('end', () => {
  try {
    const request = JSON.parse(input);
    const query = request.query || '';
    const name = query.trim() || 'World';

    const response = {
      items: [
        {
          uid: 'greeting',
          title: `Hello, ${name}!`,
          subtitle: 'A friendly greeting from the Hello World plugin',
          icon: { emoji: '👋' },
          actions: [
            { type: 'copy', text: `Hello, ${name}!`, title: 'Copy greeting' },
          ],
        },
        {
          uid: 'wave',
          title: `👋 Wave to ${name}`,
          subtitle: 'Copy a wave emoji',
          icon: { emoji: '🙌' },
          actions: [
            { type: 'copy', text: `👋 ${name}`, title: 'Copy wave' },
          ],
        },
        {
          uid: 'time',
          title: `Current time: ${new Date().toLocaleTimeString()}`,
          subtitle: 'Copy the current time',
          icon: { emoji: '🕐' },
          actions: [
            { type: 'copy', text: new Date().toLocaleTimeString(), title: 'Copy time' },
          ],
        },
      ],
    };

    process.stdout.write(JSON.stringify(response));
  } catch (err) {
    process.stderr.write(`Error: ${err.message}\n`);
    process.stdout.write(JSON.stringify({ items: [] }));
  }
});
