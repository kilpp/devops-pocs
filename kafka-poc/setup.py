from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="kafka-poc",
    version="0.1.0",
    author="Your Name",
    description="Kafka Proof of Concept - Producer and Consumer Examples",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/kilpp/devops-pocs",
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    python_requires=">=3.8",
    install_requires=[
        "kafka-python-ng==2.2.2",
    ],
    extras_require={
        "dev": [
            "pytest>=7.0",
            "pytest-cov>=4.0",
            "black>=23.0",
            "flake8>=6.0",
            "mypy>=1.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "kafka-producer=src.producer:main",
            "kafka-consumer=src.consumer:main",
        ],
    },
)
