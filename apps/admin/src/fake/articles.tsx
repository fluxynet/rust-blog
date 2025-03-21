type Article = {
    id: string;
    title: string;
    description: string;
    content: string;
    updated_at: Date;
    created_at: Date;
    deleted_at?: Date;
    status: "published" | "draft";
    author: string;
};

export const fakeArticles: Article[] = [
    {
        id: '1',
        title: 'Understanding TypeScript',
        description: 'A comprehensive guide to TypeScript.',
        content: 'TypeScript is a typed superset of JavaScript...',
        updated_at: new Date('2023-01-01T10:00:00Z'),
        created_at: new Date('2023-01-01T09:00:00Z'),
        status: "draft",
        author: 'John Doe'
    },
    {
        id: '2',
        title: 'React Hooks in Depth',
        description: 'An in-depth look at React Hooks.',
        content: 'React Hooks are functions that let you use state...',
        updated_at: new Date('2023-02-01T10:00:00Z'),
        created_at: new Date('2023-02-01T09:00:00Z'),
        status: "draft",
        author: 'Jane Smith'
    },
    {
        id: '3',
        title: 'Advanced JavaScript',
        description: 'Exploring advanced concepts in JavaScript.',
        content: 'JavaScript is a versatile language...',
        updated_at: new Date('2023-03-01T10:00:00Z'),
        created_at: new Date('2023-03-01T09:00:00Z'),
        status: "published",
        author: 'Alice Johnson'
    },
    {
        id: '4',
        title: 'CSS Grid Layout',
        description: 'A guide to CSS Grid Layout.',
        content: 'CSS Grid Layout is a two-dimensional layout system...',
        updated_at: new Date('2023-04-01T10:00:00Z'),
        created_at: new Date('2023-04-01T09:00:00Z'),
        status: "published",
        author: 'Bob Brown'
    },
    {
        id: '5',
        title: 'Node.js Performance Tips',
        description: 'Tips for improving Node.js performance.',
        content: 'Node.js is a powerful JavaScript runtime...',
        updated_at: new Date('2023-05-01T10:00:00Z'),
        created_at: new Date('2023-05-01T09:00:00Z'),
        status: "published",
        author: 'Charlie Davis'
    },
    {
        id: '6',
        title: 'GraphQL Basics',
        description: 'An introduction to GraphQL.',
        content: 'GraphQL is a query language for APIs...',
        updated_at: new Date('2023-06-01T10:00:00Z'),
        created_at: new Date('2023-06-01T09:00:00Z'),
        status: "published",
        author: 'Dana Evans'
    },
    {
        id: '7',
        title: 'Vue.js for Beginners',
        description: 'Getting started with Vue.js.',
        content: 'Vue.js is a progressive JavaScript framework...',
        updated_at: new Date('2023-07-01T10:00:00Z'),
        created_at: new Date('2023-07-01T09:00:00Z'),
        status: "published",
        author: 'Eve Foster'
    },
    {
        id: '8',
        title: 'Building REST APIs with Express',
        description: 'A guide to building REST APIs with Express.',
        content: 'Express is a minimal and flexible Node.js web application framework...',
        updated_at: new Date('2023-08-01T10:00:00Z'),
        created_at: new Date('2023-08-01T09:00:00Z'),
        deleted_at: new Date('2023-08-01T09:00:00Z'),
        status: "draft",
        author: 'Frank Green'
    },
    {
        id: '9',
        title: 'Introduction to Docker',
        description: 'Learn the basics of Docker.',
        content: 'Docker is a platform for developing, shipping, and running applications...',
        updated_at: new Date('2023-09-01T10:00:00Z'),
        created_at: new Date('2023-09-01T09:00:00Z'),
        deleted_at: new Date('2023-09-01T09:00:00Z'),
        status: "draft",
        author: 'Grace Hall'
    },
    {
        id: '10',
        title: 'Kubernetes Essentials',
        description: 'Essential concepts of Kubernetes.',
        content: 'Kubernetes is an open-source system for automating deployment...',
        updated_at: new Date('2023-10-01T10:00:00Z'),
        created_at: new Date('2025-03-20T19:50:00Z'),
        status: "draft",
        author: 'Henry Irving'
    }
];