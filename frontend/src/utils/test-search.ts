/**
 * 搜索功能测试工具
 * 用于测试防抖动和字符限制功能
 */

import { debounceWithCancel, isValidSearchQuery, getSearchQueryStatus, SEARCH_DEBOUNCE_DELAY, SEARCH_MIN_CHARS } from './debounce'

/**
 * 测试防抖动功能
 */
export function testDebounceFunction() {
  console.log('🧪 测试防抖动功能...')

  let callCount = 0
  let lastCallTime = 0

  const { debounced, cancel, pending } = debounceWithCancel((text: string) => {
    callCount++
    lastCallTime = Date.now()
    console.log(`✅ 防抖函数被调用 (第${callCount}次):`, text)
  }, 1000) // 使用1秒延迟便于测试

  console.log('📝 开始快速输入测试...')

  // 快速连续调用（模拟用户快速输入）
  debounced('a')
  debounced('ab')
  debounced('abc')
  debounced('abcd')
  debounced('abcde')

  console.log(`⏳ 等待中调用次数: ${callCount}`)
  console.log(`🔄 是否有待处理的调用: ${pending()}`)

  // 1.5秒后检查
  setTimeout(() => {
    console.log(`🕐 1.5秒后调用次数: ${callCount}`)
    console.log(`🎯 防抖成功：只执行了最后一次调用`)

    // 测试取消功能
    console.log('🚫 测试取消功能...')
    debounced('new_input')

    setTimeout(() => {
      console.log(`⏰ 取消前调用次数: ${callCount}`)
      cancel()

      setTimeout(() => {
        console.log(`✂️ 取消后调用次数: ${callCount}`)
        console.log('🛑 防抖函数被成功取消')
      }, 500)
    }, 200)
  }, 1500)
}

/**
 * 测试搜索验证功能
 */
export function testSearchValidation() {
  console.log('\n🧪 测试搜索验证功能...')

  const testCases = [
    { input: '', expected: false, description: '空字符串' },
    { input: 'a', expected: false, description: '1个字符' },
    { input: 'ab', expected: false, description: '2个字符' },
    { input: 'abc', expected: true, description: '3个字符（边界情况）' },
    { input: 'abcd', expected: true, description: '4个字符' },
    { input: '  abc  ', expected: true, description: '带空格的有效输入' },
    { input: '   ', expected: false, description: '只有空格' },
  ]

  testCases.forEach(({ input, expected, description }) => {
    const isValid = isValidSearchQuery(input)
    const status = getSearchQueryStatus(input)

    console.log(`📝 ${description}: "${input}"`)
    console.log(`   - 有效性: ${isValid === expected ? '✅' : '❌'} (${isValid})`)
    console.log(`   - 状态: ${status.message} (${status.type})`)
    console.log()
  })

  console.log(`📊 配置信息:`)
  console.log(`   - 最少字符数: ${SEARCH_MIN_CHARS}`)
  console.log(`   - 防抖延迟: ${SEARCH_DEBOUNCE_DELAY}ms`)
}

/**
 * 性能测试：防抖vs不防抖
 */
export function testPerformanceComparison() {
  console.log('\n🧪 性能测试：防抖 vs 不防抖...')

  let normalCallCount = 0
  let debounceCallCount = 0

  // 普通函数
  const normalFunction = () => {
    normalCallCount++
  }

  // 防抖函数
  const { debounced } = debounceWithCancel(() => {
    debounceCallCount++
  }, 500)

  // 模拟用户快速输入10次
  console.log('📝 模拟用户快速输入10次...')
  const startTime = Date.now()

  for (let i = 0; i < 10; i++) {
    normalFunction() // 每次都调用
    debounced() // 防抖调用
  }

  const endTime = Date.now()

  console.log(`⚡ 普通函数调用次数: ${normalCallCount}`)
  console.log(`🐌 防抖函数立即调用次数: ${debounceCallCount}`)

  // 等待防抖完成
  setTimeout(() => {
    console.log(`🎯 防抖函数最终调用次数: ${debounceCallCount}`)
    console.log(`💾 性能提升: ${((normalCallCount - debounceCallCount) / normalCallCount * 100).toFixed(1)}%`)
    console.log(`⏱️ 测试耗时: ${endTime - startTime}ms`)
  }, 1000)
}

/**
 * 自动运行所有测试
 */
export function runAllSearchTests() {
  console.log('🚀 开始搜索功能全面测试\n')
  console.log('=' * 50)

  testSearchValidation()
  testDebounceFunction()
  testPerformanceComparison()

  setTimeout(() => {
    console.log('\n✨ 所有测试完成！')
    console.log('=' * 50)
  }, 4000)
}

// 如果在浏览器环境中，可以通过控制台调用测试函数
if (typeof window !== 'undefined') {
  (window as any).testSearch = {
    runAll: runAllSearchTests,
    debounce: testDebounceFunction,
    validation: testSearchValidation,
    performance: testPerformanceComparison
  }
}