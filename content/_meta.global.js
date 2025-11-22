// {
//   "index": "About Me",
//   "contact": {
//     "title": "Contact ✉️",
//     "type": "page",
//     "href": "https://github.com/alexander-rw",
//     "newWindow": true
//   },
//   "other-writers": "Other Writers I Read"
// }


export default {
  index: {
    type: 'page'
  },
  posts: {
    type: 'page',
    items: {
      draft: {
        display: 'hidden'
      }
    }
  }
}