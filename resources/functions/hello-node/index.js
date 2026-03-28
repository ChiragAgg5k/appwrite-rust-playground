export default async ({ req, res, log }) => {
    log("Hello from Appwrite Functions!");

    return res.json({
        message: "Hello from Appwrite Functions 👋",
        mode: req.bodyText || "unknown"
    });
};
