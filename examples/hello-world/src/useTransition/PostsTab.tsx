const PostsTab = function PostsTab() {
  const items = []
  for (let i = 0; i < 50; i++) {
    items.push(<SlowPost key={i} index={i} />)
  }
  return <ul className='items'>{items}</ul>
}
function SlowPost({index}) {
  const startTime = performance.now()
  while (performance.now() - startTime < 32) {}
  return <li className='item'>博文 #{index + 1}</li>
}
export default PostsTab
